use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    input_event::InputEvent,
    ownership::{
        HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement, ReadNode,
    },
};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use std::rc::Rc;
use unicode_segmentation::UnicodeSegmentation;
// cpp: interaction/text_editor.h:18-19
pub type SyntheticEventDispatcher<'a> = Rc<dyn Fn(&mut Event, u64) + 'a>;
// cpp: interaction/text_editor.h:25-32
pub use layoutng_assembly::caret::TextCaret;
use layoutng_assembly::editing_state::ByteBoundary;
pub use layoutng_assembly::editing_state::{Selection, SelectionState};

// cpp: interaction/text_editor.h:23-43
#[derive(Default)]
pub struct TextEditor {
    // A handle to layout-owned state; editor commands do not own a second selection map.
    selections: Rc<SelectionState>,
}
// cpp: interaction/text_editor.cc:13-30
fn PreviousCodePoint(value: &str, mut offset: usize) -> usize {
    if offset == 0 {
        return 0;
    }
    offset -= 1;
    while offset > 0 && value.as_bytes()[offset] & 0xc0 == 0x80 {
        offset -= 1
    }
    offset
}
fn NextCodePoint(value: &str, mut offset: usize) -> usize {
    if offset >= value.len() {
        return value.len();
    }
    offset += 1;
    while offset < value.len() && value.as_bytes()[offset] & 0xc0 == 0x80 {
        offset += 1
    }
    offset
}

fn WordSelection(value: &str, offset: usize) -> Selection {
    if value.is_empty() {
        return Selection::default();
    }
    let offset = ByteBoundary(value, offset.min(value.len()));
    let probe = if offset == value.len() {
        PreviousCodePoint(value, offset)
    } else {
        offset
    };
    value
        .split_word_bound_indices()
        .find_map(|(start, segment)| {
            let end = start + segment.len();
            (probe >= start && probe < end).then_some(Selection {
                anchor: start,
                focus: end,
            })
        })
        .unwrap_or(Selection {
            anchor: offset,
            focus: offset,
        })
}
// cpp: interaction/text_editor.cc:33-44
fn IsTextControl(document: &InteractionDocument, id: u64) -> bool {
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
impl TextEditor {
    pub fn WithSelectionState(selections: Rc<SelectionState>) -> Self {
        Self { selections }
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
            let value = d.ControlValue(i);
            self.selections.CaretForValue(id, &value)
        })
        .flatten()
    }
    pub fn FinishComposition(&self, id: u64) {
        self.selections.SetComposition(id, None);
    }
    // cpp: interaction/text_editor.cc:49-54
    pub fn Focus(&self, document: &InteractionDocument, id: u64) {
        if !IsTextControl(document, id) {
            return;
        }
        let end = ReadNode(document, id, |d, _, i| d.ControlValue(i).len()).unwrap_or(0);
        self.selections.Set(
            id,
            Selection {
                anchor: end,
                focus: end,
            },
        );
    }
    // cpp: interaction/text_editor.cc:56-60
    pub fn SelectionFor(&self, id: u64) -> Selection {
        self.selections.Get(id).unwrap_or_default()
    }
    pub fn PlaceCaret(&self, document: &InteractionDocument, id: u64, utf16: u32, extend: bool) {
        if !IsTextControl(document, id) {
            return;
        }
        let value = ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
        let mut units = 0;
        let mut offset = value.len();
        for (byte, ch) in value.char_indices() {
            if units >= utf16 {
                offset = byte;
                break;
            }
            units += ch.len_utf16() as u32;
        }
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
        let value = ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
        let mut units = 0;
        let mut offset = value.len();
        for (byte, ch) in value.char_indices() {
            if units >= utf16 {
                offset = byte;
                break;
            }
            units += ch.len_utf16() as u32;
        }
        self.selections.Set(id, WordSelection(&value, offset));
    }
    // cpp: interaction/text_editor.cc:62-102
    fn CommitText(
        &self,
        event: &mut Event,
        document: &InteractionDocument,
        id: u64,
        replacement: String,
        input_type: String,
        emit: &InteractionDOMMutationEmitter,
        dispatch: &SyntheticEventDispatcher<'_>,
    ) -> bool {
        let mut before = MakeSyntheticEvent(EventType::kBeforeInput, id);
        before.text = replacement.clone();
        before.input_type = input_type.clone();
        dispatch(&mut before, id);
        if before.default_prevented {
            event.default_handled = true;
            return true;
        }
        let mut value = ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
        let mut selection = self.selections.Get(id).unwrap_or(Selection {
            anchor: value.len(),
            focus: value.len(),
        });
        selection.anchor = ByteBoundary(&value, selection.anchor);
        selection.focus = ByteBoundary(&value, selection.focus);
        let start = selection.Start();
        let end = selection.End();
        value.replace_range(start..end, &replacement);
        let caret = start + replacement.len();
        self.selections.Set(
            id,
            Selection {
                anchor: caret,
                focus: caret,
            },
        );
        emit(&DOMMutation {
            mutation_type: DOMMutationType::kSetControlValue,
            target_node_id: id,
            value,
            ..Default::default()
        });
        let mut input = MakeSyntheticEvent(EventType::kInput, id);
        input.text = replacement;
        input.input_type = input_type;
        dispatch(&mut input, id);
        event.default_handled = true;
        true
    }
    // cpp: interaction/text_editor.cc:104-199
    pub fn HandleEvent(
        &self,
        event: &mut Event,
        document: &InteractionDocument,
        id: u64,
        emit: &InteractionDOMMutationEmitter,
        dispatch: &SyntheticEventDispatcher<'_>,
    ) -> bool {
        if !IsTextControl(document, id) || HasAttribute(document, id, "disabled") {
            return false;
        }
        let textarea = ReadNode(document, id, |_, n, _| IsElement(n, "textarea")).unwrap_or(false);
        if HasAttribute(document, id, "readonly") {
            let mut ty = String::new();
            let mut data = String::new();
            if event.r#type == EventType::kCompositionEnd {
                ty = "insertCompositionText".into();
                data = event.text.clone()
            } else if event.r#type == EventType::kTextInput
                || (event.r#type == EventType::kKeyDown && !event.text.is_empty())
            {
                ty = "insertText".into();
                data = event.text.clone()
            } else if event.r#type == EventType::kKeyDown && event.key == "Backspace" {
                ty = "deleteContentBackward".into()
            } else if event.r#type == EventType::kKeyDown && event.key == "Delete" {
                ty = "deleteContentForward".into()
            } else if event.r#type == EventType::kKeyDown && event.key == "Enter" && textarea {
                ty = "insertLineBreak".into();
                data = "\n".into()
            }
            if !ty.is_empty() {
                let mut before = MakeSyntheticEvent(EventType::kBeforeInput, id);
                before.text = data;
                before.input_type = ty;
                dispatch(&mut before, id);
                event.default_handled = true;
                return true;
            }
            return false;
        }
        // InputMethodController::SetComposition selects and replaces the previous
        // composition, then applies the IME selection inside the inserted text.
        if event.r#type == EventType::kCompositionStart {
            self.selections
                .SetComposition(id, Some(self.SelectionFor(id)));
            event.default_handled = true;
            return true;
        }
        if matches!(
            event.r#type,
            EventType::kCompositionUpdate | EventType::kCompositionEnd
        ) {
            let value = ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
            let range = self
                .selections
                .Composition(id)
                .unwrap_or_else(|| self.SelectionFor(id));
            self.selections.Set(id, range);
            let start = ByteBoundary(&value, range.Start());
            let updated = self.CommitText(
                event,
                document,
                id,
                event.text.clone(),
                "insertCompositionText".into(),
                emit,
                dispatch,
            );
            if event.r#type == EventType::kCompositionUpdate {
                let current =
                    ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
                // A cancelled beforeinput must not create a range for text that
                // was never inserted (or overwrite a listener's selection).
                let mut expected = value.clone();
                expected.replace_range(start..ByteBoundary(&value, range.End()), &event.text);
                if current == expected {
                    self.selections.SetComposition(
                        id,
                        Some(Selection {
                            anchor: start,
                            focus: start + event.text.len(),
                        }),
                    );
                    let native = match &event.underlying_event {
                        Some(InputEvent::Composition(v)) => v.selection,
                        _ => None,
                    }
                    .unwrap_or((event.text.len(), event.text.len()));
                    self.selections.Set(
                        id,
                        Selection {
                            anchor: start + ByteBoundary(&event.text, native.0),
                            focus: start + ByteBoundary(&event.text, native.1),
                        },
                    );
                }
            } else {
                self.selections.SetComposition(id, None);
            }
            return updated;
        }
        if event.r#type == EventType::kTextInput
            && matches!(&event.underlying_event, Some(InputEvent::TextInput(_)))
        {
            return self.CommitText(
                event,
                document,
                id,
                event.text.clone(),
                "insertText".into(),
                emit,
                dispatch,
            );
        }
        if event.r#type != EventType::kKeyDown {
            return false;
        }
        let value = ReadNode(document, id, |d, _, i| d.ControlValue(i)).unwrap_or_default();
        let mut selection = self.selections.Get(id).unwrap_or(Selection {
            anchor: value.len(),
            focus: value.len(),
        });
        selection.anchor = ByteBoundary(&value, selection.anchor);
        selection.focus = ByteBoundary(&value, selection.focus);
        self.selections.Set(id, selection);
        let command = event.modifiers.control || event.modifiers.meta;
        if command && (event.key == "a" || event.key == "A") {
            self.selections.Set(
                id,
                Selection {
                    anchor: 0,
                    focus: value.len(),
                },
            );
            event.default_handled = true;
            return true;
        }
        if matches!(
            event.key.as_str(),
            "ArrowLeft" | "ArrowRight" | "Home" | "End"
        ) {
            let mut next = selection.focus;
            if event.key == "ArrowLeft" {
                next = if !event.modifiers.shift && !selection.Collapsed() {
                    selection.Start()
                } else {
                    PreviousCodePoint(&value, next)
                };
            }
            if event.key == "ArrowRight" {
                next = if !event.modifiers.shift && !selection.Collapsed() {
                    selection.End()
                } else {
                    NextCodePoint(&value, next)
                };
            }
            if event.key == "Home" {
                next = 0
            }
            if event.key == "End" {
                next = value.len()
            }
            if event.modifiers.shift {
                selection.focus = next
            } else {
                selection = Selection {
                    anchor: next,
                    focus: next,
                }
            }
            self.selections.Set(id, selection);
            event.default_handled = true;
            return true;
        }
        if event.key == "Backspace" || event.key == "Delete" {
            if selection.Collapsed() {
                if event.key == "Backspace" {
                    selection.anchor = PreviousCodePoint(&value, selection.focus)
                } else {
                    selection.focus = NextCodePoint(&value, selection.focus)
                }
            }
            self.selections.Set(id, selection);
            if selection.Collapsed() {
                event.default_handled = true;
                return true;
            }
            let ty = if event.key == "Backspace" {
                "deleteContentBackward"
            } else {
                "deleteContentForward"
            };
            return self.CommitText(
                event,
                document,
                id,
                String::new(),
                ty.into(),
                emit,
                dispatch,
            );
        }
        if event.key == "Enter" && textarea {
            return self.CommitText(
                event,
                document,
                id,
                "\n".into(),
                "insertLineBreak".into(),
                emit,
                dispatch,
            );
        }
        if !event.text.is_empty() && !command {
            return self.CommitText(
                event,
                document,
                id,
                event.text.clone(),
                "insertText".into(),
                emit,
                dispatch,
            );
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_selection_uses_unicode_word_boundaries() {
        assert_eq!(
            WordSelection("alpha beta", 7),
            Selection {
                anchor: 6,
                focus: 10,
            }
        );
        assert_eq!(
            WordSelection("alpha beta", 10),
            Selection {
                anchor: 6,
                focus: 10,
            }
        );
    }
}

use crate::{
    event::{Event, EventType, MakeSyntheticEvent},
    input_event::InputEvent,
    ownership::{
        HasAttribute, InteractionDOMMutationEmitter, InteractionDocument, IsElement, ReadNode,
    },
};
use dom::dom_mutation::{DOMMutation, DOMMutationType};
use std::rc::Rc;
// cpp: interaction/text_editor.h:18-19
pub type SyntheticEventDispatcher<'a> = Rc<dyn Fn(&mut Event, u64) + 'a>;
// cpp: interaction/text_editor.h:25-32
pub use layoutng_assembly::caret::TextCaret;
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
        selection.anchor = selection.anchor.min(value.len());
        selection.focus = selection.focus.min(value.len());
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
        if event.r#type == EventType::kCompositionEnd {
            return self.CommitText(
                event,
                document,
                id,
                event.text.clone(),
                "insertCompositionText".into(),
                emit,
                dispatch,
            );
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
        selection.anchor = selection.anchor.min(value.len());
        selection.focus = selection.focus.min(value.len());
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
                next = PreviousCodePoint(&value, next)
            }
            if event.key == "ArrowRight" {
                next = NextCodePoint(&value, next)
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

//! Persistent layout-owned editing state. Interaction writes logical positions;
//! layout/paint consume them without taking ownership of the editor commands.
#![allow(non_snake_case)]
use crate::caret::{FrameCaret, TextCaret};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

/// Control-value positions use UTF-8 byte boundaries; geometry uses UTF-16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub anchor: usize,
    pub focus: usize,
}
impl Selection {
    pub fn Start(&self) -> usize {
        self.anchor.min(self.focus)
    }
    pub fn End(&self) -> usize {
        self.anchor.max(self.focus)
    }
    pub fn Collapsed(&self) -> bool {
        self.anchor == self.focus
    }
}
#[derive(Default)]
pub struct SelectionState {
    selections: RefCell<HashMap<u64, Selection>>,
    revision: Cell<u64>,
}
impl SelectionState {
    pub fn Get(&self, id: u64) -> Option<Selection> {
        self.selections.borrow().get(&id).copied()
    }
    pub fn Set(&self, id: u64, selection: Selection) {
        if self.Get(id) != Some(selection) {
            self.selections.borrow_mut().insert(id, selection);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn Revision(&self) -> u64 {
        self.revision.get()
    }
    pub fn CaretForValue(&self, id: u64, value: &str) -> Option<TextCaret> {
        let selection = self.Get(id).unwrap_or_default();
        if !selection.Collapsed() {
            return None;
        }
        let mut offset = selection.focus.min(value.len());
        while !value.is_char_boundary(offset) {
            offset -= 1;
        }
        Some(TextCaret {
            node_id: id,
            utf16_offset: value[..offset].encode_utf16().count() as u32,
            empty: value.is_empty(),
        })
    }
}
#[derive(Default)]
pub struct LayoutEditingState {
    pub selections: Rc<SelectionState>,
    pub caret: RefCell<FrameCaret>,
}

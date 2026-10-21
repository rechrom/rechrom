//! Persistent layout-owned editing state. Interaction writes logical positions;
//! layout/paint consume them without taking ownership of the editor commands.
#![allow(non_snake_case)]
use crate::caret::{CaretPaintState, FrameCaret, TextCaret};
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

/// A DOM-text boundary expressed in the immutable layout output.  The tree
/// order is exported by LayoutNG, so Interaction and Paint can compare text
/// positions without retaining or snapshotting the DOM/LayoutObject tree.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentSelectionPosition {
    pub tree_order: u64,
    pub node_id: u64,
    pub offset: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentSelection {
    pub anchor: DocumentSelectionPosition,
    pub focus: DocumentSelectionPosition,
}

impl DocumentSelection {
    pub fn Collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    pub fn Ordered(&self) -> (DocumentSelectionPosition, DocumentSelectionPosition) {
        if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
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
    compositions: RefCell<HashMap<u64, Selection>>,
    document: Cell<Option<DocumentSelection>>,
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
    pub fn Composition(&self, id: u64) -> Option<Selection> {
        self.compositions.borrow().get(&id).copied()
    }
    pub fn SetComposition(&self, id: u64, range: Option<Selection>) {
        if self.Composition(id) == range {
            return;
        }
        let mut compositions = self.compositions.borrow_mut();
        if let Some(range) = range {
            compositions.insert(id, range);
        } else {
            compositions.remove(&id);
        }
        self.revision.set(self.revision.get().wrapping_add(1));
    }
    pub fn Document(&self) -> Option<DocumentSelection> {
        self.document.get()
    }
    pub fn SetDocument(&self, selection: Option<DocumentSelection>) {
        if self.Document() != selection {
            self.document.set(selection);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn Revision(&self) -> u64 {
        self.revision.get()
    }
    pub fn CaretForValue(&self, id: u64, value: &str) -> Option<TextCaret> {
        let selection = self.Get(id).unwrap_or_default();
        let composition = self.Composition(id);
        let mut offset = selection.focus.min(value.len());
        while !value.is_char_boundary(offset) {
            offset -= 1;
        }
        Some(TextCaret {
            node_id: id,
            utf16_offset: value[..offset].encode_utf16().count() as u32,
            empty: value.is_empty(),
            selection: (!selection.Collapsed()).then(|| {
                (
                    value[..ByteBoundary(value, selection.Start())]
                        .encode_utf16()
                        .count() as u32,
                    value[..ByteBoundary(value, selection.End())]
                        .encode_utf16()
                        .count() as u32,
                )
            }),
            composition: composition.map(|range| {
                (
                    value[..ByteBoundary(value, range.Start())]
                        .encode_utf16()
                        .count() as u32,
                    value[..ByteBoundary(value, range.End())]
                        .encode_utf16()
                        .count() as u32,
                )
            }),
        })
    }
}
#[derive(Default)]
pub struct LayoutEditingState {
    pub selections: Rc<SelectionState>,
    pub caret: RefCell<FrameCaret>,
}
impl LayoutEditingState {
    pub fn PaintState(&self) -> EditingPaintState {
        EditingPaintState {
            caret: self.caret.borrow().paint_state(),
            document_selection: self.selections.Document(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditingPaintState {
    pub caret: Option<CaretPaintState>,
    pub document_selection: Option<DocumentSelection>,
}

/// Clamp stale/script/native offsets without slicing into a Unicode code point.
pub fn ByteBoundary(value: &str, offset: usize) -> usize {
    let mut offset = offset.min(value.len());
    while !value.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

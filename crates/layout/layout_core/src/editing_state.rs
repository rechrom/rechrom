//! Persistent layout-owned editing state. Interaction writes logical positions;
//! layout/paint consume them without taking ownership of the editor commands.
#![allow(non_snake_case)]
use crate::caret::{CaretPaintState, DragCaret, FrameCaret, TextCaret};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

/// Control-value positions and geometry use source UTF-16 code-unit offsets.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Selection {
    pub anchor: usize,
    pub focus: usize,
}

/// Immutable Paint projection of the active TextControlElement selection.
/// SelectionState remains the sole owner of the cached offsets; Paint receives
/// only the author host identity and ordered UTF-16 range needed to match the
/// anonymous inner-editor text fragments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextControlSelectionPaintState {
    pub host_id: u64,
    pub start: u32,
    pub end: u32,
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

/// Canonical FrameSelection endpoints. Neighbor anchors and affinity survive
/// independently of the text-only Layout/Paint selection projection below.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DOMSelection {
    pub anchor: crate::EditingPosition,
    pub focus: crate::EditingPosition,
    pub affinity: crate::EditingTextAffinity,
}

/// Immutable per-Text Paint projection of DocumentMarkerController state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentCompositionMarker {
    pub node_id: u64,
    pub start: u32,
    pub end: u32,
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
    document_composition_markers: RefCell<Rc<[DocumentCompositionMarker]>>,
    pub layout_selection: RefCell<crate::layout_selection::LayoutSelection>,
    selections: RefCell<HashMap<u64, Selection>>,
    compositions: RefCell<HashMap<u64, Selection>>,
    document: Cell<Option<DocumentSelection>>,
    dom_selection: Cell<Option<DOMSelection>>,
    // A control keeps its cached offsets after the FrameSelection moves away.
    // This discriminator identifies the current selection without duplicating
    // any endpoint or interpreting focus as selection ownership.
    active_control: Cell<Option<u64>>,
    selection_handles_visible: Cell<bool>,
    revision: Cell<u64>,
}
impl SelectionState {
    pub fn SetDocumentCompositionMarkers(&self, ranges: Rc<[DocumentCompositionMarker]>) {
        if *self.document_composition_markers.borrow() != ranges {
            *self.document_composition_markers.borrow_mut() = ranges;
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn Get(&self, id: u64) -> Option<Selection> {
        self.selections.borrow().get(&id).copied()
    }
    pub fn Set(&self, id: u64, selection: Selection) {
        if self.Get(id) != Some(selection)
            || self.active_control.get() != Some(id)
            || self.dom_selection.get().is_some()
        {
            self.layout_selection.borrow_mut().SetHasPendingSelection();
            self.selections.borrow_mut().insert(id, selection);
            self.active_control.set(Some(id));
            self.document.set(None);
            self.dom_selection.set(None);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    /// TextControlElement's cached selection survives changes to the frame's
    /// active selection. Programmatic/reset updates must not take frame focus.
    pub fn SetCachedControlSelection(&self, id: u64, selection: Selection) {
        if self.Get(id) != Some(selection) {
            self.selections.borrow_mut().insert(id, selection);
            if self.active_control.get() == Some(id) {
                self.layout_selection.borrow_mut().SetHasPendingSelection();
            }
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    /// TextControlElement::selectionChanged caches UTF-16 offsets after
    /// FrameSelection has published its real inner-editor DOM endpoints. Keep
    /// both views on this one owner: the DOM endpoints remain canonical while
    /// the active-control discriminator identifies the author host for Paint
    /// and the text-control WebIDL cache.
    pub fn SetActiveControlSelection(&self, id: u64, selection: Selection) {
        if self.Get(id) != Some(selection) || self.active_control.get() != Some(id) {
            self.layout_selection.borrow_mut().SetHasPendingSelection();
            self.selections.borrow_mut().insert(id, selection);
            self.active_control.set(Some(id));
            self.document.set(None);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn ActiveControl(&self) -> Option<u64> {
        self.active_control.get()
    }
    pub fn TextControlPaintSelection(&self) -> Option<TextControlSelectionPaintState> {
        let host_id = self.ActiveControl()?;
        let selection = self.Get(host_id)?;
        (!selection.Collapsed()).then_some(TextControlSelectionPaintState {
            host_id,
            start: selection.Start().min(u32::MAX as usize) as u32,
            end: selection.End().min(u32::MAX as usize) as u32,
        })
    }
    pub fn Clear(&self) {
        if self.active_control.take().is_some()
            || self.dom_selection.take().is_some()
            || self.document.get().is_some()
        {
            self.layout_selection.borrow_mut().SetHasPendingSelection();
            self.document.set(None);
            self.dom_selection.set(None);
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
    pub fn ClearCompositions(&self) {
        let mut compositions = self.compositions.borrow_mut();
        if compositions.is_empty() {
            return;
        }
        compositions.clear();
        self.revision.set(self.revision.get().wrapping_add(1));
    }
    pub fn Document(&self) -> Option<DocumentSelection> {
        self.document.get()
    }
    pub fn DOMSelection(&self) -> Option<DOMSelection> {
        self.dom_selection.get()
    }
    pub fn SetDOMSelection(
        &self,
        selection: Option<DOMSelection>,
        paint: Option<DocumentSelection>,
    ) {
        if self.dom_selection.get() != selection
            || self.document.get() != paint
            || self.active_control.get().is_some()
        {
            self.layout_selection.borrow_mut().SetHasPendingSelection();
            self.dom_selection.set(selection);
            self.document.set(paint);
            self.active_control.set(None);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn SetDocument(&self, selection: Option<DocumentSelection>) {
        // Legacy text selections already carry canonical DOM text endpoints.
        // Production FrameSelection calls SetDOMSelection directly; it never
        // reconstructs source positions from this derived paint value.
        if let Some(value) = selection {
            self.SetDOMSelection(
                Some(DOMSelection {
                    anchor: crate::EditingPosition::new(
                        value.anchor.node_id,
                        value.anchor.offset as usize,
                    ),
                    focus: crate::EditingPosition::new(
                        value.focus.node_id,
                        value.focus.offset as usize,
                    ),
                    affinity: crate::EditingTextAffinity::Downstream,
                }),
                selection,
            );
        } else if self.dom_selection.get().is_some() {
            self.SetDOMSelection(None, None);
        }
    }
    pub fn SelectionHandlesVisible(&self) -> bool {
        self.selection_handles_visible.get()
    }
    pub fn SetSelectionHandlesVisible(&self, visible: bool) {
        if self.SelectionHandlesVisible() != visible {
            self.selection_handles_visible.set(visible);
            self.revision.set(self.revision.get().wrapping_add(1));
        }
    }
    pub fn Revision(&self) -> u64 {
        self.revision.get()
    }
    pub fn CaretForValue(&self, id: u64, value: &str) -> Option<TextCaret> {
        self.CaretForUTF16Value(id, &value.encode_utf16().collect::<Vec<_>>())
    }
    pub fn CaretForUTF16Value(&self, id: u64, value: &[u16]) -> Option<TextCaret> {
        let selection = self.Get(id).unwrap_or_default();
        let composition = self.Composition(id);
        Some(TextCaret {
            native: None,
            control_block_text: false,
            node_id: id,
            text_control_host: Some(id),
            utf16_offset: selection.focus.min(value.len()).min(u32::MAX as usize) as u32,
            empty: value.is_empty(),
            selection: (!selection.Collapsed()).then(|| {
                (
                    selection.Start().min(value.len()).min(u32::MAX as usize) as u32,
                    selection.End().min(value.len()).min(u32::MAX as usize) as u32,
                )
            }),
            composition: composition.map(|range| {
                (
                    range.Start().min(value.len()).min(u32::MAX as usize) as u32,
                    range.End().min(value.len()).min(u32::MAX as usize) as u32,
                )
            }),
        })
    }
}
#[derive(Default)]
pub struct LayoutEditingState {
    pub selections: Rc<SelectionState>,
    pub caret: RefCell<FrameCaret>,
    pub drag_caret: RefCell<DragCaret>,
}
impl LayoutEditingState {
    pub fn PaintState(&self) -> EditingPaintState {
        EditingPaintState {
            document_composition_markers: self
                .selections
                .document_composition_markers
                .borrow()
                .clone(),
            caret: self.caret.borrow().paint_state(),
            drag_caret: self.drag_caret.borrow().paint_state(),
            text_control_selection: self.selections.TextControlPaintSelection(),
            document_selection: self
                .selections
                .layout_selection
                .borrow()
                .DocumentProjection(),
            selection_handles_visible: self.selections.SelectionHandlesVisible(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EditingPaintState {
    pub document_composition_markers: Rc<[DocumentCompositionMarker]>,
    pub caret: Option<CaretPaintState>,
    pub drag_caret: Option<CaretPaintState>,
    pub text_control_selection: Option<TextControlSelectionPaintState>,
    pub document_selection: Option<DocumentSelection>,
    pub selection_handles_visible: bool,
}

/// Clamp stale/script/source offsets without slicing into a Unicode code point.
pub fn ByteBoundary(value: &str, offset: usize) -> usize {
    let mut offset = offset.min(value.len());
    while !value.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

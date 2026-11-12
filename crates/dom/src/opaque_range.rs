//! Chromium 6c1d401f core/dom/opaque_range.cc24-144 and
//! html/forms/text_control_element.cc1393-1548. The TextControl retains the
//! actual OpaqueRange owners; offsets are value UTF-16 indices, never a second
//! DOM Range or a projection of the current selection.
use crate::{
    error::{DOMException, DOMExceptionKind},
    persistent_document::{DOMInputType, DOMNodeType, DOMOwnerHandle, PersistentDocument},
    range::{DOMBoundaryPoint, DOMStaticRange},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct DOMOpaqueRangeHandle(Rc<OpaqueRangeData>);
struct OpaqueRangeData {
    owner_document: DOMOwnerHandle,
    element: Cell<Option<DOMOwnerHandle>>,
    start: Cell<u32>,
    end: Cell<u32>,
    supplement: Weak<RefCell<TextControlOpaqueRanges>>,
}
impl PartialEq for DOMOpaqueRangeHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for DOMOpaqueRangeHandle {}
impl std::fmt::Debug for DOMOpaqueRangeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpaqueRange")
            .field("owner_document", &self.OwnerDocument())
            .field("element", &self.GetElement())
            .field("start", &self.startOffset())
            .field("end", &self.endOffset())
            .finish()
    }
}
impl DOMOpaqueRangeHandle {
    pub fn startOffset(&self) -> u32 {
        self.0.start.get()
    }
    pub fn endOffset(&self) -> u32 {
        self.0.end.get()
    }
    pub fn collapsed(&self) -> bool {
        self.startOffset() == self.endOffset()
    }
    pub fn startContainer(&self) -> Option<DOMOwnerHandle> {
        None
    }
    pub fn endContainer(&self) -> Option<DOMOwnerHandle> {
        None
    }
    pub fn IsStaticRange(&self) -> bool {
        false
    }
    pub fn IsOpaqueRange(&self) -> bool {
        true
    }
    pub fn OwnerDocument(&self) -> DOMOwnerHandle {
        self.0.owner_document
    }
    pub fn GetElement(&self) -> Option<DOMOwnerHandle> {
        self.0.element.get()
    }
    pub fn disconnect(&self) {
        if self.GetElement().is_some() {
            if let Some(supplement) = self.0.supplement.upgrade() {
                let mut supplement = supplement.borrow_mut();
                if let Some(index) = supplement.ranges.iter().position(|range| range == self) {
                    supplement.ranges.remove(index);
                }
            }
            self.0.element.set(None);
        }
        self.0.start.set(0);
        self.0.end.set(0);
    }
    /// Original unsigned arithmetic and early insertion/deletion branches.
    fn UpdateOffsetsForTextChange(
        &self,
        change_offset: u32,
        deleted_count: u32,
        inserted_count: u32,
        value_length: u32,
    ) {
        debug_assert!(foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled());
        if self.GetElement().is_none() || (deleted_count == 0 && inserted_count == 0) {
            return;
        }
        let (pre_start, pre_end) = (self.startOffset(), self.endOffset());
        if deleted_count == 0 && inserted_count > 0 {
            if pre_start == pre_end && change_offset == pre_start {
                return;
            }
            if pre_start != pre_end && change_offset == pre_start {
                self.0.end.set(pre_end.wrapping_add(inserted_count));
                return;
            }
            if pre_start != pre_end && change_offset == pre_end {
                return;
            }
        }
        if deleted_count > 0 && change_offset == 0 && pre_end <= deleted_count {
            self.0.start.set(0);
            self.0.end.set(0);
            return;
        }
        let change_end = change_offset.wrapping_add(deleted_count);
        let calculate_new_offset = |pos: u32| {
            if pos < change_offset {
                pos
            } else if pos < change_end {
                change_offset
            } else {
                pos.wrapping_sub(deleted_count).wrapping_add(inserted_count)
            }
        };
        let new_start = calculate_new_offset(pre_start).min(value_length);
        let mut new_end = calculate_new_offset(pre_end).min(value_length);
        if new_start > new_end {
            new_end = new_start;
        }
        self.0.start.set(new_start);
        self.0.end.set(new_end);
    }
}
#[derive(Default)]
pub(crate) struct TextControlOpaqueRanges {
    ranges: Vec<DOMOpaqueRangeHandle>,
    pending_user_edit: Option<PendingUserEditSnapshot>,
    skip_next_set_value_auto_diff: bool,
}
struct PendingUserEditSnapshot {
    old_value: Rc<[u16]>,
    selection_start: u32,
    selection_end: u32,
}
fn SourceLength(length: usize) -> u32 {
    u32::try_from(length).expect("Blink String length fits unsigned")
}
impl PersistentDocument {
    /// HTMLInputElement.cc2429-2445 plus inherited TextControl createValueRange.
    pub fn CreateValueRange(
        &mut self,
        element: usize,
        start: u32,
        mut end: u32,
    ) -> Result<DOMOpaqueRangeHandle, DOMException> {
        assert!(foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled());
        if let Some(kind) = self.InputType(element) {
            if !matches!(
                kind,
                DOMInputType::kText
                    | DOMInputType::kSearch
                    | DOMInputType::kURL
                    | DOMInputType::kTelephone
                    | DOMInputType::kPassword
            ) {
                return Err(DOMException { kind: DOMExceptionKind::NotSupportedError, message: "<input> element must be of a text field type: text, search, url, tel, or password." });
            }
        } else if !self.Node(element).IsHTMLElement("textarea") {
            return Err(DOMException {
                kind: DOMExceptionKind::InvalidNodeType,
                message: "createValueRange requires a text control",
            });
        }
        let length = SourceLength(self.ControlValueUTF16(element).len());
        if start > length || end > length {
            return Err(DOMException {
                kind: DOMExceptionKind::IndexSizeError,
                message: "Start or end offset exceeds value length.",
            });
        }
        if start > end {
            end = start;
        }
        self.CountUse(crate::use_counter::WebFeature::kOpaqueRange);
        let supplement = self.TextControlOpaqueRangeState(element);
        let range = DOMOpaqueRangeHandle(Rc::new(OpaqueRangeData {
            owner_document: self.NodeDocumentHandle(element),
            element: Cell::new(Some(self.NodeHandle(element))),
            start: Cell::new(start),
            end: Cell::new(end),
            supplement: Rc::downgrade(&supplement),
        }));
        supplement.borrow_mut().ranges.push(range.clone());
        Ok(range)
    }
    pub fn DisconnectAllOpaqueRanges(&self, element: usize) {
        let Some(supplement) = self.ExistingTextControlOpaqueRangeState(element) else {
            return;
        };
        loop {
            let range = supplement.borrow().ranges.last().cloned();
            let Some(range) = range else {
                break;
            };
            range.disconnect();
        }
    }
    pub fn HasOpaqueRanges(&self, element: usize) -> bool {
        self.ExistingTextControlOpaqueRangeState(element)
            .is_some_and(|s| !s.borrow().ranges.is_empty())
    }
    pub fn CaptureOpaqueRangePreEdit(
        &mut self,
        element: usize,
        selection_start: usize,
        selection_end: usize,
    ) {
        debug_assert!(foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled());
        if !self.HasOpaqueRanges(element) {
            return;
        }
        let old_value = self.TextControlInnerEditorValueUTF16(element);
        let old_length = old_value.len();
        self.TextControlOpaqueRangeState(element)
            .borrow_mut()
            .pending_user_edit = Some(PendingUserEditSnapshot {
            old_value,
            selection_start: SourceLength(selection_start.min(old_length)),
            selection_end: SourceLength(selection_end.min(old_length)),
        });
    }
    pub fn CommitOpaqueRangeEdit(&self, element: usize) {
        debug_assert!(foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled());
        let Some(supplement) = self.ExistingTextControlOpaqueRangeState(element) else {
            return;
        };
        let pending = supplement.borrow_mut().pending_user_edit.take();
        if supplement.borrow().ranges.is_empty() {
            return;
        }
        let Some(pending) = pending else {
            return;
        };
        self.ApplyOpaqueRangeUpdate(
            element,
            &pending.old_value,
            pending.selection_start,
            pending.selection_end,
        );
    }
    pub fn CommitProgrammaticOpaqueRangeEdit(
        &self,
        element: usize,
        old_value: &[u16],
        old_start: usize,
        old_end: usize,
    ) {
        if !foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled()
            || !self.HasOpaqueRanges(element)
        {
            return;
        }
        self.ExistingTextControlOpaqueRangeState(element)
            .unwrap()
            .borrow_mut()
            .pending_user_edit
            .take();
        self.ApplyOpaqueRangeUpdate(
            element,
            old_value,
            SourceLength(old_start),
            SourceLength(old_end),
        );
    }
    fn ApplyOpaqueRangeUpdate(
        &self,
        element: usize,
        old_value: &[u16],
        sel_start: u32,
        sel_end: u32,
    ) {
        let new_value = self.TextControlInnerEditorValueUTF16(element);
        if old_value == new_value.as_ref() {
            return;
        }
        let old_length = old_value.len();
        let new_length = new_value.len();
        let mut selection_start = (sel_start as usize).min(old_length);
        let mut selection_end = (sel_end as usize).min(old_length);
        if selection_start > selection_end {
            std::mem::swap(&mut selection_start, &mut selection_end);
        }
        let mut prefix = 0;
        while prefix < old_length && prefix < new_length && old_value[prefix] == new_value[prefix] {
            prefix += 1;
        }
        prefix = prefix.min(selection_start);
        let suffix_limit =
            (old_length - prefix).min((new_length - prefix).min(old_length - selection_end));
        let mut suffix = 0;
        while suffix < suffix_limit
            && old_value[old_length - 1 - suffix] == new_value[new_length - 1 - suffix]
        {
            suffix += 1;
        }
        let deleted_count = old_length - prefix - suffix;
        let inserted_count = new_length - prefix - suffix;
        if deleted_count != 0 || inserted_count != 0 {
            self.NotifyOpaqueRangesOfTextChange(
                element,
                SourceLength(prefix),
                SourceLength(deleted_count),
                SourceLength(inserted_count),
            );
        }
    }
    pub fn NotifyOpaqueRangesOfTextChange(
        &self,
        element: usize,
        change: u32,
        deleted: u32,
        inserted: u32,
    ) {
        debug_assert!(foundation::RuntimeEnabledFeatures::OpaqueRangeEnabled());
        let Some(supplement) = self.ExistingTextControlOpaqueRangeState(element) else {
            return;
        };
        if supplement.borrow().ranges.is_empty() {
            return;
        }
        let value_length = SourceLength(self.ControlValueUTF16(element).len());
        for range in &supplement.borrow().ranges {
            range.UpdateOffsetsForTextChange(change, deleted, inserted, value_length);
        }
    }
    pub fn SetSkipNextSetValueAutoDiff(&mut self, element: usize, skip: bool) {
        self.TextControlOpaqueRangeState(element)
            .borrow_mut()
            .skip_next_set_value_auto_diff = skip;
    }
    pub fn ShouldSkipNextSetValueAutoDiff(&self, element: usize) -> bool {
        self.ExistingTextControlOpaqueRangeState(element)
            .is_some_and(|s| s.borrow().skip_next_set_value_auto_diff)
    }
    /// TextControlElement.cc960-990. This is deliberately distinct from the
    /// general PlainTextRange resolver: only immediate Text/BR children count.
    pub fn ResolveTextControlValueOffset(
        &self,
        element: usize,
        target: u32,
    ) -> Option<DOMBoundaryPoint> {
        let editor = self.TextControlInnerEditorElement(element)?;
        let mut offset: u32 = 0;
        let mut last_text = None;
        for &node in self.Node(editor).Children() {
            if self.Node(node).Type() == DOMNodeType::kText {
                let length = SourceLength(self.CharacterDataUTF16(node).len());
                let node_end = offset.wrapping_add(length);
                if target <= node_end {
                    return Some(DOMBoundaryPoint {
                        node_id: self.Node(node).Id(),
                        offset: target.wrapping_sub(offset) as usize,
                    });
                }
                last_text = Some(node);
                offset = node_end;
            } else if self.Node(node).IsHTMLElement("br")
                && !self.TextControlIsPlaceholderBreak(node)
            {
                if let Some(last) = last_text {
                    if target <= offset {
                        return Some(DOMBoundaryPoint {
                            node_id: self.Node(last).Id(),
                            offset: self.CharacterDataUTF16(last).len(),
                        });
                    }
                }
                offset = offset.wrapping_add(1);
            }
        }
        last_text.map(|node| DOMBoundaryPoint {
            node_id: self.Node(node).Id(),
            offset: self.CharacterDataUTF16(node).len(),
        })
    }
    /// Called only after the source JavaScript style/layout barrier and real
    /// HasLayoutObject query; the returned range borrows no layout geometry.
    pub fn ResolveOpaqueRangeForValue(
        &self,
        range: &DOMOpaqueRangeHandle,
    ) -> Option<DOMStaticRange> {
        let element = range.GetElement()?;
        let node = self.FindNodeById(element.Id())?;
        if self.NodeHandle(node) != element {
            return None;
        }
        Some(DOMStaticRange {
            start: self.ResolveTextControlValueOffset(node, range.startOffset())?,
            end: self.ResolveTextControlValueOffset(node, range.endOffset())?,
        })
    }
}

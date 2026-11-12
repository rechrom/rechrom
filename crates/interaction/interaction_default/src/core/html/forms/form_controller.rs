//! Chromium 6c1d401fcca5e1b0030563a90c2f2bba168e0c15.
//! core/html/forms/form_controller.{h,cc}. The retained document state and
//! restoration queue share the live DOM/control owners through a narrow port.
use super::input_type::EventQueueScope;
use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    rc::Rc,
};
pub type StateString = Rc<[u16]>;
pub type StateVector = Vec<StateString>;
fn text(value: &str) -> StateString {
    value.encode_utf16().collect::<Vec<_>>().into()
}
fn signature() -> StateString {
    text("\n\r?% Blink serialized form state version 10 \n\r=&")
}
fn number(value: usize) -> StateString {
    text(&value.to_string())
}
fn uint(value: &[u16]) -> usize {
    crate::platform::wtf::text::string_to_number::StringToUintLoose(value).unwrap_or(0) as usize
}
fn valid_custom_element_name(name: &[u16]) -> bool {
    let Ok(name) = String::from_utf16(name) else {
        return false;
    };
    name.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && name.contains('-')
        && !name.bytes().any(|c| {
            c.is_ascii_uppercase()
                || matches!(c, 0 | b'\t' | b'\n' | b'\x0c' | b'\r' | b' ' | b'/' | b'>')
        })
        && !matches!(
            name.as_str(),
            "annotation-xml"
                | "color-profile"
                | "font-face"
                | "font-face-src"
                | "font-face-uri"
                | "font-face-format"
                | "font-face-name"
                | "missing-glyph"
        )
}
/// Individual ListedElement/Form/Document facts. No style, layout, parser or
/// application owner crosses into the controller; control values stay live.
pub trait FormStateDocument {
    fn StatefulControlRevision(&self) -> u64;
    fn StatefulControls(&self) -> Vec<u64>;
    fn ListedControls(&self, form: u64) -> Vec<u64>;
    fn OwnerFormForState(&self, control: u64) -> Option<u64>;
    fn FormSignature(&self, form: u64) -> StateString;
    fn ControlName(&self, control: u64) -> StateString;
    fn ControlType(&self, control: u64) -> StateString;
    fn ShouldSaveAndRestore(&self, control: u64) -> bool;
    fn SaveControlState(&self, control: u64) -> StateVector;
    fn RestoreControlState(
        &self,
        control: u64,
        state: &[StateString],
    ) -> Result<(), navigation_history::FormStateRestoreError>;
    fn IsDisabledOrReadonly(&self, control: u64) -> bool;
    fn IsDirtyControl(&self, control: u64) -> bool;
    fn HasFinishedParsing(&self) -> bool;
    fn IsActive(&self) -> bool;
    fn BeginEventQueueScope(&self) -> Box<dyn EventQueueScope + '_>;
    fn PostRestoreTask(&self, task: Box<dyn FnOnce()>);
}
#[derive(Default)]
struct FormKeyGenerator {
    forms: HashMap<u64, StateString>,
    indices: HashMap<StateString, usize>,
}
impl FormKeyGenerator {
    fn FormKey(&mut self, document: &dyn FormStateDocument, control: u64) -> StateString {
        let Some(form) = document.OwnerFormForState(control) else {
            return text("No owner");
        };
        if let Some(key) = self.forms.get(&form) {
            return key.clone();
        }
        let signature = document.FormSignature(form);
        let index = self.indices.entry(signature.clone()).or_default();
        let mut key = signature.to_vec();
        key.extend(" #".encode_utf16());
        key.extend(index.to_string().encode_utf16());
        *index += 1;
        let key: StateString = key.into();
        self.forms.insert(form, key.clone());
        key
    }
}
#[derive(Default)]
struct SavedFormState {
    states: HashMap<(StateString, StateString), VecDeque<StateVector>>,
    count: usize,
}
impl SavedFormState {
    fn Append(&mut self, name: StateString, kind: StateString, state: StateVector) {
        self.states
            .entry((name, kind))
            .or_default()
            .push_back(state);
        self.count += 1;
    }
    fn Take(&mut self, name: StateString, kind: StateString) -> StateVector {
        let key = (name, kind);
        let Some(queue) = self.states.get_mut(&key) else {
            return Vec::new();
        };
        let state = queue.pop_front().expect("nonempty source state queue");
        self.count -= 1;
        if queue.is_empty() {
            self.states.remove(&key);
        }
        state
    }
    fn SerializeTo(&self, output: &mut StateVector) {
        output.push(number(self.count));
        for ((name, kind), queue) in &self.states {
            for state in queue {
                output.push(name.clone());
                output.push(kind.clone());
                output.push(number(state.len()));
                output.extend(state.iter().cloned());
            }
        }
    }
    fn Deserialize(input: &[StateString], index: &mut usize) -> Option<Self> {
        let count = uint(input.get(*index)?);
        *index += 1;
        if count == 0 {
            return None;
        }
        let mut result = Self::default();
        for _ in 0..count {
            let name = input.get(*index)?.clone();
            let kind = input.get(index.checked_add(1)?)?.clone();
            *index += 2;
            if kind.is_empty() || kind.iter().any(|&c| c != 45 && !(97..=122).contains(&c)) {
                // Native controls have ASCII type names. Custom element names
                // are validated by the same source CustomElement name parser.
                if !valid_custom_element_name(&kind) {
                    return None;
                }
            }
            let size = uint(input.get(*index)?);
            *index += 1;
            let end = index.checked_add(size)?;
            let state = input.get(*index..end)?.to_vec();
            *index = end;
            result.Append(name, kind, state);
        }
        Some(result)
    }
}
fn deserialize(input: &[StateString]) -> HashMap<StateString, SavedFormState> {
    let mut result = HashMap::new();
    if input.first() != Some(&signature()) {
        return result;
    }
    let mut index = 1;
    while index + 1 < input.len() {
        let key = input[index].clone();
        index += 1;
        let Some(state) = SavedFormState::Deserialize(input, &mut index) else {
            return HashMap::new();
        };
        result.entry(key).or_insert(state);
    }
    if index != input.len() {
        result.clear();
    }
    result
}
/// This is HistoryItem's lazy owner, not a serialized snapshot. Its control
/// list is invalidated only by the document's stateful-control membership epoch.
pub struct DocumentState {
    document: Rc<dyn FormStateDocument>,
    controls: RefCell<(Option<u64>, Vec<u64>)>,
}
impl DocumentState {
    fn GetControlList(&self) -> std::cell::Ref<'_, Vec<u64>> {
        let revision = self.document.StatefulControlRevision();
        let mut controls = self.controls.borrow_mut();
        if controls.0 != Some(revision) {
            *controls = (Some(revision), self.document.StatefulControls());
        }
        drop(controls);
        std::cell::Ref::map(self.controls.borrow(), |controls| &controls.1)
    }
    pub fn InvalidateControlList(&self) {
        *self.controls.borrow_mut() = (None, Vec::new());
    }
}
impl navigation_history::DocumentState for DocumentState {
    fn ToStateVector(&self) -> StateVector {
        let mut keys = FormKeyGenerator::default();
        let mut forms: HashMap<StateString, SavedFormState> = HashMap::new();
        for &control in self.GetControlList().iter() {
            if !self.document.ShouldSaveAndRestore(control) {
                continue;
            }
            forms
                .entry(keys.FormKey(self.document.as_ref(), control))
                .or_default()
                .Append(
                    self.document.ControlName(control),
                    self.document.ControlType(control),
                    self.document.SaveControlState(control),
                );
        }
        if forms.is_empty() {
            return Vec::new();
        }
        let mut result = vec![signature()];
        for (key, form) in forms {
            result.push(key);
            form.SerializeTo(&mut result);
        }
        result
    }
}
pub struct FormController {
    document: Rc<dyn FormStateDocument>,
    state: Rc<DocumentState>,
    saved: RefCell<HashMap<StateString, SavedFormState>>,
    keys: RefCell<FormKeyGenerator>,
    did_restore_all: Cell<bool>,
    restore_error: RefCell<Option<navigation_history::FormStateRestoreError>>,
}
impl FormController {
    pub fn new(document: Rc<dyn FormStateDocument>) -> Rc<Self> {
        Rc::new(Self {
            state: Rc::new(DocumentState {
                document: document.clone(),
                controls: RefCell::new((None, Vec::new())),
            }),
            document,
            saved: RefCell::new(HashMap::new()),
            keys: RefCell::new(FormKeyGenerator::default()),
            did_restore_all: Cell::new(false),
            restore_error: RefCell::new(None),
        })
    }
    pub fn ControlStates(&self) -> Rc<dyn navigation_history::DocumentState> {
        self.state.clone()
    }
    pub fn InvalidateStatefulFormControlList(&self) {
        self.state.InvalidateControlList();
    }
    pub fn SetStateForNewControls(&self, input: &[StateString]) {
        *self.saved.borrow_mut() = deserialize(input);
    }
    pub fn HasControlStates(&self) -> bool {
        !self.saved.borrow().is_empty()
    }
    pub fn WillDeleteForm(&self, form: u64) {
        self.keys.borrow_mut().forms.remove(&form);
    }
    fn TakeStateForControl(&self, control: u64) -> StateVector {
        if !self.HasControlStates() {
            return Vec::new();
        }
        let key = self
            .keys
            .borrow_mut()
            .FormKey(self.document.as_ref(), control);
        let mut saved = self.saved.borrow_mut();
        let Some(form) = saved.get_mut(&key) else {
            return Vec::new();
        };
        let state = form.Take(
            self.document.ControlName(control),
            self.document.ControlType(control),
        );
        if form.states.is_empty() {
            saved.remove(&key);
        }
        state
    }
    fn RestoreControlStateInternal(&self, control: u64) {
        if !self.document.ShouldSaveAndRestore(control) {
            return;
        }
        let state = self.TakeStateForControl(control);
        if state.is_empty()
            || self.document.IsDisabledOrReadonly(control)
            || self.document.IsDirtyControl(control)
        {
            return;
        }
        self.RestoreState(control, state);
    }
    fn RestoreState(&self, control: u64, state: StateVector) {
        if let Err(error) = self.document.RestoreControlState(control, &state) {
            let key = self
                .keys
                .borrow_mut()
                .FormKey(self.document.as_ref(), control);
            let mut saved = self.saved.borrow_mut();
            let form = saved.entry(key).or_default();
            form.states
                .entry((
                    self.document.ControlName(control),
                    self.document.ControlType(control),
                ))
                .or_default()
                .push_front(state);
            form.count += 1;
            *self.restore_error.borrow_mut() = Some(error);
        }
    }
    pub fn RestoreError(&self) -> Option<navigation_history::FormStateRestoreError> {
        self.restore_error.borrow().clone()
    }
    pub fn RestoreControlStateFor(&self, control: u64) {
        if self.document.HasFinishedParsing() && self.document.OwnerFormForState(control).is_none()
        {
            self.RestoreControlStateInternal(control);
        }
    }
    pub fn RestoreControlStateIn(&self, form: u64) {
        if !self.document.HasFinishedParsing() {
            return;
        }
        let _scope = self.document.BeginEventQueueScope();
        for control in self.document.ListedControls(form) {
            if self.document.OwnerFormForState(control) == Some(form) {
                self.RestoreControlStateInternal(control);
                if self.RestoreError().is_some() {
                    break;
                }
            }
        }
    }
    pub fn RestoreControlStateOnUpgrade(&self, control: u64) {
        if !self.document.ShouldSaveAndRestore(control) {
            return;
        }
        let state = self.TakeStateForControl(control);
        if !state.is_empty() {
            self.RestoreState(control, state);
        }
    }
    pub fn ScheduleRestore(self: &Rc<Self>) {
        let controller = Rc::downgrade(self);
        self.document.PostRestoreTask(Box::new(move || {
            if let Some(controller) = controller.upgrade() {
                controller.RestoreAllControlsInDocumentOrder();
            }
        }));
    }
    pub fn RestoreImmediately(&self) {
        self.restore_error.borrow_mut().take();
        if !self.did_restore_all.get() && self.HasControlStates() {
            self.RestoreAllControlsInDocumentOrder();
        }
    }
    fn RestoreAllControlsInDocumentOrder(&self) {
        if !self.document.IsActive() || self.did_restore_all.get() {
            return;
        }
        let mut finished = HashSet::new();
        let _scope = self.document.BeginEventQueueScope();
        let controls = self.state.GetControlList().clone();
        for control in controls {
            match self.document.OwnerFormForState(control) {
                None => self.RestoreControlStateFor(control),
                Some(form) if finished.insert(form) => self.RestoreControlStateIn(form),
                _ => {}
            }
            if self.RestoreError().is_some() {
                return;
            }
        }
        self.did_restore_all.set(true);
    }
    pub fn GetReferencedFilePaths(input: &[StateString]) -> StateVector {
        let mut paths = Vec::new();
        for form in deserialize(input).into_values() {
            for ((_, kind), queue) in form.states {
                if kind.as_ref() == [102, 105, 108, 101] {
                    for state in queue {
                        paths.extend(super::file_input_type::FilesFromFormControlState(&state));
                    }
                }
            }
        }
        paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use navigation_history::DocumentState as _;
    struct Scope;
    impl EventQueueScope for Scope {}
    struct Document {
        values: RefCell<Vec<StateVector>>,
        restored: RefCell<Vec<(u64, StateVector)>>,
        unavailable: Cell<bool>,
    }
    impl Document {
        fn new(values: &[&str]) -> Rc<Self> {
            Rc::new(Self {
                values: RefCell::new(values.iter().map(|value| vec![text(value)]).collect()),
                restored: RefCell::new(Vec::new()),
                unavailable: Cell::new(false),
            })
        }
    }
    impl FormStateDocument for Document {
        fn StatefulControlRevision(&self) -> u64 {
            0
        }
        fn StatefulControls(&self) -> Vec<u64> {
            (0..self.values.borrow().len() as u64).collect()
        }
        fn ListedControls(&self, _: u64) -> Vec<u64> {
            self.StatefulControls()
        }
        fn OwnerFormForState(&self, _: u64) -> Option<u64> {
            None
        }
        fn FormSignature(&self, _: u64) -> StateString {
            unreachable!()
        }
        fn ControlName(&self, _: u64) -> StateString {
            text("repeated")
        }
        fn ControlType(&self, _: u64) -> StateString {
            text("text")
        }
        fn ShouldSaveAndRestore(&self, _: u64) -> bool {
            true
        }
        fn SaveControlState(&self, control: u64) -> StateVector {
            self.values.borrow()[control as usize].clone()
        }
        fn RestoreControlState(
            &self,
            control: u64,
            state: &[StateString],
        ) -> Result<(), navigation_history::FormStateRestoreError> {
            if self.unavailable.get() {
                return Err(navigation_history::FormStateRestoreError::NativeFileReaderUnavailable);
            }
            self.restored.borrow_mut().push((control, state.to_vec()));
            Ok(())
        }
        fn IsDisabledOrReadonly(&self, _: u64) -> bool {
            false
        }
        fn IsDirtyControl(&self, _: u64) -> bool {
            false
        }
        fn HasFinishedParsing(&self) -> bool {
            true
        }
        fn IsActive(&self) -> bool {
            true
        }
        fn BeginEventQueueScope(&self) -> Box<dyn EventQueueScope + '_> {
            Box::new(Scope)
        }
        fn PostRestoreTask(&self, _: Box<dyn FnOnce()>) {
            unreachable!("tests use immediate restoration")
        }
    }
    #[test]
    fn history_keeps_live_values_and_duplicate_controls_restore_in_order() {
        let document = Document::new(&["first", "second"]);
        let controller = FormController::new(document.clone());
        let item = navigation_history::HistoryItemOwner::Create();
        item.SetDocumentStateOwner(controller.ControlStates());
        document.values.borrow_mut()[0] = vec![text("edited after attachment")];
        let next = Document::new(&["default", "default"]);
        let restore = FormController::new(next.clone());
        restore.SetStateForNewControls(&item.DocumentState());
        restore.RestoreImmediately();
        assert_eq!(
            *next.restored.borrow(),
            vec![
                (0, vec![text("edited after attachment")]),
                (1, vec![text("second")])
            ]
        );
    }
    #[test]
    fn skipped_default_keeps_duplicate_matching_and_invalid_vectors_are_atomic() {
        let document = Document::new(&["first", "second"]);
        document.values.borrow_mut()[0].clear();
        let controller = FormController::new(document);
        let state = controller.state.ToStateVector();
        let next = Document::new(&["default", "default"]);
        let restore = FormController::new(next.clone());
        let mut corrupt = state.clone();
        corrupt.push(text("trailing form key"));
        restore.SetStateForNewControls(&corrupt);
        assert!(!restore.HasControlStates());
        restore.SetStateForNewControls(&state);
        restore.RestoreImmediately();
        assert_eq!(*next.restored.borrow(), vec![(1, vec![text("second")])]);
    }
    #[test]
    fn failed_host_admission_retains_fifo_state_for_retry() {
        let document = Document::new(&["first", "second"]);
        let saved = FormController::new(document).state.ToStateVector();
        let next = Document::new(&["default", "default"]);
        next.unavailable.set(true);
        let restore = FormController::new(next.clone());
        restore.SetStateForNewControls(&saved);
        restore.RestoreImmediately();
        assert!(restore.HasControlStates());
        assert!(restore.RestoreError().is_some());
        assert!(next.restored.borrow().is_empty());
        next.unavailable.set(false);
        restore.RestoreImmediately();
        assert_eq!(
            *next.restored.borrow(),
            vec![(0, vec![text("first")]), (1, vec![text("second")])]
        );
        assert!(!restore.HasControlStates());
    }
}

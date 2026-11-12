//! Document's FormController attachment. The controller algorithm and source
//! per-control virtuals live in Interaction; DOM retains only a weak attachment
//! so HistoryItem's lazy DocumentState can retain the live document without an
//! Rc cycle.
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub trait DocumentFormController {
    fn ControlStates(&self) -> Rc<dyn navigation_history::DocumentState>;
    fn SetStateForNewControls(&self, state: &[Rc<[u16]>]);
    fn RestoreImmediately(&self) -> Result<(), navigation_history::FormStateRestoreError>;
    fn SetNativeFileReader(&self, reader: std::sync::Arc<dyn fileapi::NativeFileReader>);
    fn RestoreError(&self) -> Option<navigation_history::FormStateRestoreError>;
    fn StatefulControlConnected(&self, node: u64);
    fn CustomElementUpgraded(&self, node: u64);
    fn FinishedParsing(
        &self,
        node: u64,
        kind: crate::attribute_lifecycle::FormStateParsingKind,
        was_finished: bool,
    );
    fn DocumentFinishedParsing(&self);
    fn WillDeleteForm(&self, node: u64);
}
impl crate::PersistentDocument {
    pub fn AttachFormController(&self, controller: Weak<dyn DocumentFormController>) {
        *self.form_controller.borrow_mut() = Some(controller);
    }
    pub fn FormController(&self) -> Option<Rc<dyn DocumentFormController>> {
        self.form_controller
            .borrow()
            .as_ref()
            .and_then(Weak::upgrade)
    }
    pub fn FormElementsState(&self) -> Rc<dyn navigation_history::DocumentState> {
        self.FormController()
            .expect("Document FormController installed before history commit")
            .ControlStates()
    }
    pub fn SetStateForNewFormElements(&self, state: &[Rc<[u16]>]) {
        self.FormController()
            .expect("Document FormController installed before form restoration")
            .SetStateForNewControls(state);
    }
}
/// Restore can mutate controls and dispatch scoped events; release the DOM
/// borrow before invoking the source controller.
pub fn RestoreFormControlStateImmediately(
    document: &Rc<RefCell<crate::DOM>>,
) -> Result<(), navigation_history::FormStateRestoreError> {
    let controller = document.borrow().GetDocument().FormController();
    if let Some(controller) = controller {
        controller.RestoreImmediately()?;
    }
    Ok(())
}

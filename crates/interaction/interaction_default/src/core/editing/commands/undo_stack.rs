// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: core/editing/commands/undo_stack.cc:45-169.
use super::undo_step::{UndoStep, UndoStepContext};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
const MAXIMUM_UNDO_STACK_DEPTH: usize = 1000;
pub trait UndoStackContext<S: Clone>: UndoStepContext<S> {
    fn SetHasUndoStack(&self, element: &Self::Node, value: bool);
    fn StepDocumentInDesignMode(&self, step: &UndoStep<S>) -> bool;
}
/// Registration reads only the ending root and sets its original element
/// flag; callers need not implement unrelated history-event replay leaves.
pub trait UndoStackRegistrationContext<S: Clone> {
    type Node;
    fn RootEditableElement(&self, selection: &S) -> Option<Self::Node>;
    fn SetHasUndoStack(&self, element: &Self::Node, value: bool);
}
impl<S: Clone, C: UndoStackContext<S>> UndoStackRegistrationContext<S> for C {
    type Node = <C as UndoStepContext<S>>::Node;
    fn RootEditableElement(&self, selection: &S) -> Option<Self::Node> {
        UndoStepContext::RootEditableElement(self, selection)
    }
    fn SetHasUndoStack(&self, element: &Self::Node, value: bool) {
        UndoStackContext::SetHasUndoStack(self, element, value)
    }
}
/// The Editor's sole history owner. Stacks retain command/selection identity;
/// no DOM inverse snapshots or replacement tree are stored here.
pub struct UndoStack<S: Clone> {
    undo: RefCell<Vec<Rc<UndoStep<S>>>>,
    redo: RefCell<Vec<Rc<UndoStep<S>>>>,
    in_redo: Cell<bool>,
}
impl<S: Clone> Default for UndoStack<S> {
    fn default() -> Self {
        Self {
            undo: RefCell::new(Vec::new()),
            redo: RefCell::new(Vec::new()),
            in_redo: Cell::new(false),
        }
    }
}
struct RedoScope<'a>(&'a Cell<bool>);
impl Drop for RedoScope<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
impl<S: Clone> UndoStack<S> {
    pub fn RegisterUndoStep<C: UndoStackRegistrationContext<S>>(
        &self,
        context: &C,
        step: Rc<UndoStep<S>>,
    ) {
        {
            let mut undo = self.undo.borrow_mut();
            debug_assert!(undo
                .last()
                .is_none_or(|last| step.SequenceNumber() >= last.SequenceNumber()));
            if undo.len() == MAXIMUM_UNDO_STACK_DEPTH {
                undo.remove(0);
            }
            undo.push(step.clone());
        }
        if !self.in_redo.get() {
            self.redo.borrow_mut().clear();
        }
        self.DidSetEndingSelection(context, &step);
    }
    pub fn RegisterRedoStep(&self, step: Rc<UndoStep<S>>) {
        self.redo.borrow_mut().push(step);
    }
    pub fn CanUndo(&self) -> bool {
        !self.undo.borrow().is_empty()
    }
    pub fn CanRedo(&self) -> bool {
        !self.redo.borrow().is_empty()
    }
    // Source UndoStepRange::begin is reverse iteration (undo_stack.h:64-71).
    // Retain only the queried original step, releasing the stack guard before
    // beforeinput can change history or synchronously navigate.
    pub fn LastUndoStep(&self) -> Option<Rc<UndoStep<S>>> {
        self.undo.borrow().last().cloned()
    }
    pub fn LastRedoStep(&self) -> Option<Rc<UndoStep<S>>> {
        self.redo.borrow().last().cloned()
    }
    /// One original owner at a time for UndoStepRange's reverse iteration.
    pub fn UndoStepFromNewest(&self, index: usize) -> Option<Rc<UndoStep<S>>> {
        let undo = self.undo.borrow();
        undo.len()
            .checked_sub(index + 1)
            .and_then(|index| undo.get(index))
            .cloned()
    }
    pub fn Undo<C: UndoStepContext<S>>(&self, context: &C) {
        let step = self.undo.borrow_mut().pop();
        if let Some(step) = step {
            step.Unapply(context);
        }
    }
    pub fn Redo<C: UndoStepContext<S>>(&self, context: &C) {
        let step = self.redo.borrow_mut().pop();
        let Some(step) = step else {
            return;
        };
        assert!(
            !self.in_redo.replace(true),
            "source undo stack forbids recursive redo"
        );
        let _scope = RedoScope(&self.in_redo);
        step.Reapply(context);
    }
    pub fn Clear(&self) {
        self.undo.borrow_mut().clear();
        self.redo.borrow_mut().clear();
    }
    pub fn DidSetEndingSelection<C: UndoStackRegistrationContext<S>>(
        &self,
        context: &C,
        step: &UndoStep<S>,
    ) {
        if let Some(element) = context.RootEditableElement(&step.EndingSelection()) {
            context.SetHasUndoStack(&element, true);
        }
    }
    pub fn ElementRemoved<C: UndoStackContext<S>>(&self, context: &C, element: &C::Node) {
        // Release history borrows before any context query.
        let first_undo = self.undo.borrow().first().cloned();
        let first_redo = self.redo.borrow().first().cloned();
        if first_undo
            .as_ref()
            .is_some_and(|step| context.StepDocumentInDesignMode(step))
            || first_redo
                .as_ref()
                .is_some_and(|step| context.StepDocumentInDesignMode(step))
        {
            return;
        }
        self.undo
            .borrow_mut()
            .retain(|step| !step.IsOwnedBy(context, element));
        self.redo
            .borrow_mut()
            .retain(|step| !step.IsOwnedBy(context, element));
        UndoStackContext::SetHasUndoStack(context, element, false);
    }
    /// Command-local inspection retains each existing step; callbacks must not
    /// hold a RefCell guard over history mutation or synchronous DOM reentry.
    pub fn UndoSteps(&self) -> Vec<Rc<UndoStep<S>>> {
        self.undo.borrow().clone()
    }
    pub fn RedoSteps(&self) -> Vec<Rc<UndoStep<S>>> {
        self.redo.borrow().clone()
    }
}

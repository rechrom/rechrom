// Copyright The Chromium Authors. BSD-style license; see Chromium LICENSE.
// Chromium: core/editing/commands/undo_step.cc:30-169; undo_step.h:57-127.
use super::simple_edit_commands::SimpleEditCommand;
use crate::core::editing::set_selection_options::SetSelectionOptions;
use interaction_dispatch::core::{
    dom::events::scoped_event_queue::EventQueueScope, events::input_event::InputType,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::atomic::{AtomicU64, Ordering},
};

static CURRENT_SEQUENCE_NUMBER: AtomicU64 = AtomicU64::new(0);

/// Effects on the same document/frame/editor owners. Selection correction and
/// command lifecycle remain source operations, not DOM Range undo.
pub trait UndoStepContext<S: Clone> {
    type Node: Clone + PartialEq;
    fn UpdateStyleAndLayout(&self);
    fn EventQueueScope(&self) -> EventQueueScope;
    fn RootEditableElement(&self, selection: &S) -> Option<Self::Node>;
    fn DispatchEditableContentChangedEvents(
        &self,
        starting: Option<&Self::Node>,
        ending: Option<&Self::Node>,
    );
    fn DispatchHistoryInput(
        &self,
        starting: Option<&Self::Node>,
        ending: Option<&Self::Node>,
        input_type: InputType,
    );
    fn CorrectedSelectionAfterCommand(&self, selection: &S) -> S;
    fn ChangeSelectionAfterCommand(&self, selection: &S, options: SetSelectionOptions);
    fn ClearLastEditCommand(&self);
    fn RegisterRedoStep(&self, step: Rc<UndoStep<S>>);
    fn RegisterUndoStep(&self, step: Rc<UndoStep<S>>);
    fn RespondToChangedContentsForFrameSelection(&self);
}

/// Both lanes are raw SelectionForUndoStep values; the legacy lane may have
/// been VP-canonicalized at command birth, while the DOM lane never is.
pub struct UndoStep<S: Clone> {
    starting_selection: RefCell<S>,
    ending_selection: RefCell<S>,
    starting_dom_selection: RefCell<S>,
    ending_dom_selection: RefCell<S>,
    commands: RefCell<Vec<Rc<dyn SimpleEditCommand>>>,
    sequence_number: u64,
    selection_is_directional: Cell<bool>,
}
impl<S: Clone> UndoStep<S> {
    pub fn new(starting: S, ending: S, starting_dom: S, ending_dom: S) -> Self {
        Self {
            starting_selection: RefCell::new(starting),
            ending_selection: RefCell::new(ending),
            starting_dom_selection: RefCell::new(starting_dom),
            ending_dom_selection: RefCell::new(ending_dom),
            commands: RefCell::new(Vec::new()),
            sequence_number: CURRENT_SEQUENCE_NUMBER.fetch_add(1, Ordering::Relaxed) + 1,
            selection_is_directional: Cell::new(false),
        }
    }
    pub fn StartingSelection(&self) -> S {
        self.starting_selection.borrow().clone()
    }
    pub fn EndingSelection(&self) -> S {
        self.ending_selection.borrow().clone()
    }
    pub fn StartingDomSelection(&self) -> S {
        self.starting_dom_selection.borrow().clone()
    }
    pub fn EndingDomSelection(&self) -> S {
        self.ending_dom_selection.borrow().clone()
    }
    pub fn SetStartingSelection(&self, value: S) {
        *self.starting_selection.borrow_mut() = value;
    }
    pub fn SetEndingSelection(&self, value: S) {
        *self.ending_selection.borrow_mut() = value;
    }
    pub fn SetStartingDomSelection(&self, value: S) {
        *self.starting_dom_selection.borrow_mut() = value;
    }
    pub fn SetEndingDomSelection(&self, value: S) {
        *self.ending_dom_selection.borrow_mut() = value;
    }
    pub fn SequenceNumber(&self) -> u64 {
        self.sequence_number
    }
    pub fn SelectionIsDirectional(&self) -> bool {
        self.selection_is_directional.get()
    }
    pub fn SetSelectionIsDirectional(&self, value: bool) {
        self.selection_is_directional.set(value);
    }
    pub fn Append(&self, command: Rc<dyn SimpleEditCommand>) {
        self.commands.borrow_mut().push(command);
    }
    pub fn AppendStep(&self, step: &Self) {
        let commands = step.commands.borrow().clone();
        self.commands.borrow_mut().extend(commands);
    }
    pub fn IsOwnedBy<C: UndoStepContext<S>>(&self, context: &C, element: &C::Node) -> bool {
        context
            .RootEditableElement(&self.EndingSelection())
            .as_ref()
            == Some(element)
    }
    pub fn Unapply<C: UndoStepContext<S>>(self: &Rc<Self>, context: &C) {
        context.UpdateStyleAndLayout();
        // Retain command owners before invoking DOM/script reentry; no history
        // borrow crosses a source elementary command.
        let commands = self.commands.borrow().clone();
        for command in commands.iter().rev() {
            command.DoUnapply();
        }
        self.DidReplay(context, false);
    }
    pub fn Reapply<C: UndoStepContext<S>>(self: &Rc<Self>, context: &C) {
        context.UpdateStyleAndLayout();
        let commands = self.commands.borrow().clone();
        for command in commands {
            command.DoReapply();
        }
        self.DidReplay(context, true);
    }
    fn DidReplay<C: UndoStepContext<S>>(self: &Rc<Self>, context: &C, redo: bool) {
        let _scope = context.EventQueueScope();
        let starting = context.RootEditableElement(&self.StartingSelection());
        let ending = context.RootEditableElement(&self.EndingSelection());
        context.DispatchEditableContentChangedEvents(starting.as_ref(), ending.as_ref());
        context.DispatchHistoryInput(
            starting.as_ref(),
            ending.as_ref(),
            if redo {
                InputType::kHistoryRedo
            } else {
                InputType::kHistoryUndo
            },
        );
        let selection = context.CorrectedSelectionAfterCommand(&if redo {
            self.EndingSelection()
        } else {
            self.StartingSelection()
        });
        context.ChangeSelectionAfterCommand(
            &selection,
            SetSelectionOptions {
                should_close_typing: true,
                should_clear_typing_style: true,
                is_directional: self.SelectionIsDirectional(),
                ..Default::default()
            },
        );
        context.ClearLastEditCommand();
        if redo {
            context.RegisterUndoStep(self.clone());
        } else {
            context.RegisterRedoStep(self.clone());
        }
        // Focus listeners may have changed the selection during the setter.
        context.RespondToChangedContentsForFrameSelection();
    }
}

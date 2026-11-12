//! Document's native state-preserving move scope. Chromium document.h's
//! StatePreservingAtomicMoveInProgress and Node::moveBefore's scoped flag.
use crate::persistent_document::PersistentDocument;
use std::{cell::Cell, rc::Rc};

pub struct StatePreservingAtomicMoveScope(Rc<Cell<bool>>);
impl Drop for StatePreservingAtomicMoveScope {
    fn drop(&mut self) {
        self.0.set(false);
    }
}
impl PersistentDocument {
    pub fn StatePreservingAtomicMoveInProgress(&self) -> bool {
        self.state_preserving_atomic_move_in_progress.get()
    }
    pub fn EnterStatePreservingAtomicMove(&self) -> StatePreservingAtomicMoveScope {
        assert!(
            !self.state_preserving_atomic_move_in_progress.replace(true),
            "state-preserving atomic moves do not nest"
        );
        StatePreservingAtomicMoveScope(self.state_preserving_atomic_move_in_progress.clone())
    }
}

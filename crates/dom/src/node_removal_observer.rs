//! Script-free input notification at Document's canonical pre-unlink stage.
//!
//! Chromium: core/dom/document.cc:6431-6478;
//! core/input/event_handler.h:87-95.

use crate::persistent_document::PersistentDocument;

/// The document calls this once for the connected subtree root after Range's
/// pre-removal fixup and before tree/slot identities change. Implementations
/// may inspect the borrowed live document and update retained input state, but
/// must not invoke script or retain the document borrow.
pub trait DocumentNodeRemovalObserver {
    fn NodeWillBeRemoved(&self, document: &PersistentDocument, node: usize);

    /// Element::RemovedFrom calls EventHandler::ElementRemoved after the node
    /// has left the connected tree. This is distinct from the pre-unlink
    /// NodeWillBeRemoved callback above.
    fn ElementRemoved(&self, _document: &PersistentDocument, _node: usize) {}
}

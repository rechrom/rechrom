//! Retained Rust ownership at the source const-Document callback boundary.
//! Node identities are used instead of arena addresses; no DOM/state/editor
//! borrow survives an external callback. No DOM snapshot is used for behavior.
use dom::{
    persistent_document::{DOMNamespace, DOMNode},
    Document, DOM,
};
use std::{cell::RefCell, rc::Rc};
pub type InteractionDocument = Rc<RefCell<DOM>>;
pub type InteractionDOMMutationEmitter = Rc<dyn Fn(&dom::dom_mutation::DOMMutation)>;
pub fn ReadNode<R>(
    document: &InteractionDocument,
    id: u64,
    read: impl FnOnce(&Document, &DOMNode, usize) -> R,
) -> Option<R> {
    let owner = document.borrow();
    let doc = owner.GetDocument();
    let i = doc.FindNodeById(id)?;
    Some(read(doc, doc.Node(i), i))
}
pub fn IsElement(node: &DOMNode, name: &str) -> bool {
    node.IsElement(name, DOMNamespace::kHTML)
}
pub fn Attribute(document: &InteractionDocument, id: u64, name: &str) -> String {
    ReadNode(document, id, |_, n, _| {
        n.FindAttribute(name)
            .map_or(String::new(), |a| a.value.clone())
    })
    .unwrap_or_default()
}
pub fn HasAttribute(document: &InteractionDocument, id: u64, name: &str) -> bool {
    ReadNode(document, id, |_, n, _| n.FindAttribute(name).is_some()).unwrap_or(false)
}

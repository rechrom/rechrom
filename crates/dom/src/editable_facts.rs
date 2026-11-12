//! Live facts for the single shared Chromium editable-level query.
use crate::persistent_document::PersistentDocument;
use foundation::editable::{EditableDOM, UserModify};
pub struct DOMEditableFacts<'a>(pub &'a PersistentDocument);
impl EditableDOM for DOMEditableFacts<'_> {
    type Node = u64;
    fn IsActive(&self) -> bool {
        self.0.IsActive()
    }
    fn Parent(&self, node: u64) -> Option<u64> {
        self.0
            .Node(self.0.FindNodeById(node)?)
            .Parent()
            .map(|node| self.0.Node(node).Id())
    }
    fn IsPseudoElement(&self, _: u64) -> bool {
        false
    } // DOMNode identities exclude computed pseudo owners.
    fn IsElement(&self, node: u64) -> bool {
        self.0.FindNodeById(node).is_some_and(|node| {
            self.0.Node(node).Type() == crate::persistent_document::DOMNodeType::kElement
        })
    }
    fn IsHTMLElementOrDocument(&self, node: u64) -> bool {
        self.0.FindNodeById(node).is_some_and(|node| {
            let node = self.0.Node(node);
            node.Type() == crate::persistent_document::DOMNodeType::kDocument
                || (node.Type() == crate::persistent_document::DOMNodeType::kElement
                    && node.Namespace() == crate::persistent_document::DOMNamespace::kHTML)
        })
    }
    fn IsBody(&self, node: u64) -> bool {
        self.0
            .FindNodeById(node)
            .is_some_and(|node| self.0.Node(node).IsHTMLElement("body"))
    }
    fn IsUnassignedChildOfShadowHost(&self, node: u64) -> bool {
        self.0.FindNodeById(node).is_some_and(|node| {
            self.0
                .Node(node)
                .Parent()
                .is_some_and(|parent| self.0.ShadowRoot(parent).is_some())
                && self.0.AssignedSlot(node).is_none()
        })
    }
    fn UsedUserModify(&self, node: u64) -> Option<UserModify> {
        let node = self.0.FindNodeById(node)?;
        Some(
            match self
                .0
                .ResolvedStyleFor(node)?
                .ComputedStyle()?
                .UsedUserModify()
            {
                foundation::EUserModify::kReadOnly => UserModify::ReadOnly,
                foundation::EUserModify::kReadWrite => UserModify::ReadWrite,
                foundation::EUserModify::kReadWritePlaintextOnly => {
                    UserModify::ReadWritePlaintextOnly
                }
            },
        )
    }
    fn IsInertRoot(&self, node: u64) -> bool {
        self.0
            .FindNodeById(node)
            .is_some_and(|node| self.0.Node(node).FindAttribute("inert").is_some())
    }
}

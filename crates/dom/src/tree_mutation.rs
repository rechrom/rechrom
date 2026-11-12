// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
//! ContainerNode removal transactions across the shared canonical arena borrow.
//! Source: container_node.cc:995-1032,1105-1155,1408-1423. Source class
//! callbacks finish synchronously before the unlink commit, with no DOM borrow
//! across native event-target/task/platform entry. Page does no tree traversal.
use crate::{
    dom_mutation::{DOMMutation, DOMMutationError, DOMMutationType, ValidateDOMMutation},
    persistent_document::{DOMNodeType, PersistentDocument},
    DOM,
};
use std::{cell::RefCell, rc::Rc};

/// Same source algorithms support an exclusively borrowed parser arena and
/// the script-visible shared arena; only the primitive borrow boundary differs.
pub(crate) trait TreeMutation {
    fn Read<R>(&self, read: impl FnOnce(&PersistentDocument) -> R) -> R;
    fn Write<R>(&mut self, write: impl FnOnce(&mut PersistentDocument) -> R) -> R;
    fn RemoveNode(&mut self, node: usize);
}
impl TreeMutation for PersistentDocument {
    fn Read<R>(&self, read: impl FnOnce(&PersistentDocument) -> R) -> R {
        read(self)
    }
    fn Write<R>(&mut self, write: impl FnOnce(&mut PersistentDocument) -> R) -> R {
        write(self)
    }
    fn RemoveNode(&mut self, node: usize) {
        self.Remove(node);
    }
}
pub(crate) struct SharedTree<'a>(pub &'a Rc<RefCell<DOM>>);
impl TreeMutation for SharedTree<'_> {
    fn Read<R>(&self, read: impl FnOnce(&PersistentDocument) -> R) -> R {
        read(self.0.borrow().GetDocument())
    }
    fn Write<R>(&mut self, write: impl FnOnce(&mut PersistentDocument) -> R) -> R {
        write(self.0.borrow_mut().GetDocumentMut())
    }
    fn RemoveNode(&mut self, node: usize) {
        let preparation = self.Read(|d| {
            let parent = d.Node(node).Parent()?;
            Some((
                d.RootHandle(),
                parent,
                d.RemovalDescendants(node),
                d.AttributeLifecycle().clone(),
                d.TrackChildRemoval(node),
            ))
        });
        let Some((document, parent, descendants, lifecycle, _removal)) = preparation else {
            return;
        };
        // This is the removal walk's existing source identity list, also used
        // by the commit. It is not another tree model or a document-wide scan.
        for &element in &descendants {
            let id = self.Read(|d| {
                assert_eq!(
                    d.RootHandle(),
                    document,
                    "removal keeps the same document arena"
                );
                d.Node(element).Id()
            });
            lifecycle.WillRemove(id);
        }
        self.Write(|d| {
            assert_eq!(
                d.RootHandle(),
                document,
                "removal keeps the same document arena"
            );
            d.RemoveWithDescendants(node, parent, descendants);
        });
    }
}

/// The canonical mutation entry used by DocumentEngine. Fragment parsing only
/// allocates the fragment; replacement/removal uses this same transaction.
pub fn ApplyMutation(
    owner: &Rc<RefCell<DOM>>,
    mutation: &DOMMutation,
    parse_fragment: impl FnOnce(
        &mut PersistentDocument,
        &DOMMutation,
    ) -> Result<(usize, usize), DOMMutationError>,
) -> Result<(), DOMMutationError> {
    let mut tree = SharedTree(owner);
    tree.Read(|d| ValidateDOMMutation(d, mutation, true))?;
    let target = tree.Read(|d| {
        d.FindNodeById(mutation.target_node_id)
            .expect("validated target")
    });
    match mutation.mutation_type {
        DOMMutationType::kSetPopoverOpen => {
            let lifecycle = tree.Read(|d| d.AttributeLifecycle().clone());
            lifecycle.SetPopoverOpen(
                mutation.target_node_id,
                mutation.bool_value,
                (mutation.source_node_id != 0).then_some(mutation.source_node_id),
            )?;
        }
        DOMMutationType::kSetDialogModal => {
            let lifecycle = tree.Read(|d| d.AttributeLifecycle().clone());
            lifecycle.SetDialogModal(
                mutation.target_node_id,
                mutation.bool_value,
                mutation.utf16_value.as_deref(),
            )?;
        }
        DOMMutationType::kRemoveChild => {
            let child = tree.Read(|d| d.FindNodeById(mutation.child_node_id).unwrap());
            tree.RemoveNode(child);
        }
        DOMMutationType::kAppendChild | DOMMutationType::kInsertBefore => {
            let (children, before) = tree.Read(|d| {
                let child = d.FindNodeById(mutation.child_node_id).unwrap();
                (
                    if d.Node(child).Type() == DOMNodeType::kDocumentFragment {
                        d.Node(child).Children().to_vec()
                    } else {
                        vec![child]
                    },
                    d.FindNodeById(mutation.before_node_id),
                )
            });
            for child in children {
                if Some(child) == before {
                    continue;
                }
                tree.RemoveNode(child);
                tree.Write(|d| {
                    if let Some(before) = before {
                        d.InsertBefore(target, child, before);
                    } else {
                        d.AppendChild(target, child);
                    }
                });
            }
        }
        DOMMutationType::kSetTextContent => {
            let children = tree.Read(|d| {
                if matches!(
                    d.Node(target).Type(),
                    DOMNodeType::kElement | DOMNodeType::kDocumentFragment
                ) {
                    d.Node(target).Children().to_vec()
                } else {
                    Vec::new()
                }
            });
            for child in children {
                tree.RemoveNode(child);
            }
            tree.Write(|d| d.ApplyTextContentMutation(target, mutation));
        }
        DOMMutationType::kSetInnerHTML => {
            let (target, fragment) = tree.Write(|d| parse_fragment(d, mutation))?;
            loop {
                let child = tree.Read(|d| d.Node(target).Children().last().copied());
                let Some(child) = child else {
                    break;
                };
                tree.RemoveNode(child);
            }
            // The parser fragment is disconnected. Its ordinary source
            // insertion performs no connected removal lifecycle.
            tree.Write(|d| d.TakeAllChildren(fragment, target));
        }
        DOMMutationType::kDeleteRangeContents => {
            crate::range::DeleteContents(&mut tree, mutation.range.as_ref().unwrap())?;
        }
        DOMMutationType::kInsertRangeNode => {
            crate::range::InsertNode(
                &mut tree,
                mutation.range.as_ref().unwrap(),
                mutation.child_node_id,
            )?;
        }
        _ => tree.Write(|d| {
            crate::dom_mutation::TryApplyDOMMutations(d, std::slice::from_ref(mutation))
        })?,
    }
    Ok(())
}
impl PersistentDocument {
    fn ApplyTextContentMutation(&mut self, target: usize, mutation: &DOMMutation) {
        if let Some(units) = &mutation.utf16_value {
            self.SetTextContentUTF16(target, units);
        } else {
            self.SetTextContent(target, mutation.value.clone());
        }
    }
}

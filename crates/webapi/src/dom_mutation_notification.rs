use super::*;
use dom::mutation_observers::{MutationObserverRecipient, MutationType};

enum RecordData {
    Attributes {
        name: String,
        namespace: String,
        old_value: Option<String>,
    },
    CharacterData {
        old_value: Option<String>,
    },
    ChildList {
        children: Vec<HostObjectId>,
    },
}
struct PendingRecord {
    node: usize,
    recipients: Vec<MutationObserverRecipient>,
    data: RecordData,
}

// Pending mutation records retain removed nodes before the host commits DOM
// edge removals. Publish these roots before old edges are released, then drop
// them only after the traced NodeLists have reached JavaScript.
struct PendingRoots {
    graph: Rc<DOMHostGraph>,
    reason: u64,
    nodes: Vec<HostObjectId>,
}
impl Drop for PendingRoots {
    fn drop(&mut self) {
        for &node in &self.nodes {
            self.graph.SetRoot(node, self.reason, false);
        }
    }
}

/// Only data required by an interested mutation record crosses the mutation.
/// DOM owns registration/ancestry; JavaScript owns records and delivery jobs.
pub struct MutationNotification {
    records: Vec<PendingRecord>,
    roots: Option<PendingRoots>,
}

impl DOMJavaScriptBindings {
    // cpp: MutationObserverInterestGroup::CreateFor*Mutation/CreateIfNeeded.
    pub fn PrepareMutationNotification(
        &self,
        mutation: &DOMMutation,
    ) -> Option<MutationNotification> {
        if !self.runtime_realm.IsValid() || !self.method_wrappers.contains_key("__mutationRecord") {
            return None;
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let target = document.FindNodeById(mutation.target_node_id)?;
        let mut records = Vec::<PendingRecord>::new();
        match mutation.mutation_type {
            DOMMutationType::kSetAttribute | DOMMutationType::kRemoveAttribute => {
                let recipients = document.MutationObserverInterest(
                    target,
                    MutationType::Attributes,
                    Some((&mutation.name, &mutation.namespace_uri)),
                );
                if recipients.is_empty() {
                    return None;
                }
                let attribute = document.Node(target).Attributes().iter().find(|attribute| {
                    attribute.local_name == mutation.name
                        && attribute.namespace_uri == mutation.namespace_uri
                });
                if mutation.mutation_type == DOMMutationType::kRemoveAttribute
                    && attribute.is_none()
                {
                    return None;
                }
                let old_value = recipients
                    .iter()
                    .any(|r| r.old_value)
                    .then(|| attribute.map(|attribute| attribute.value.clone()))
                    .flatten();
                records.push(PendingRecord {
                    node: target,
                    recipients,
                    data: RecordData::Attributes {
                        name: mutation.name.clone(),
                        namespace: mutation.namespace_uri.clone(),
                        old_value,
                    },
                });
            }
            DOMMutationType::kSetTextContent
                if matches!(
                    document.Node(target).Type(),
                    DOMNodeType::kText
                        | DOMNodeType::kComment
                        | DOMNodeType::kProcessingInstruction
                ) =>
            {
                let recipients =
                    document.MutationObserverInterest(target, MutationType::CharacterData, None);
                if recipients.is_empty() {
                    return None;
                }
                let old_value = recipients
                    .iter()
                    .any(|r| r.old_value)
                    .then(|| document.Node(target).Data().to_owned());
                records.push(PendingRecord {
                    node: target,
                    recipients,
                    data: RecordData::CharacterData { old_value },
                });
            }
            _ => {
                if !document.HasMutationObserversOfType(MutationType::ChildList) {
                    return None;
                }
                let mut add = |node: Option<usize>| {
                    let Some(node) = node else {
                        return;
                    };
                    if records.iter().any(|record| record.node == node) {
                        return;
                    }
                    let recipients =
                        document.MutationObserverInterest(node, MutationType::ChildList, None);
                    if recipients.is_empty() {
                        return;
                    }
                    records.push(PendingRecord {
                        node,
                        recipients,
                        data: RecordData::ChildList {
                            children: document
                                .Node(node)
                                .Children()
                                .iter()
                                .map(|&child| document.Node(child).Id())
                                .collect(),
                        },
                    });
                };
                match mutation.mutation_type {
                    DOMMutationType::kSetTextContent => add(Some(target)),
                    DOMMutationType::kSetInnerHTML => {
                        add(if document.Node(target).IsHTMLElement("template") {
                            document.TemplateContents(target)
                        } else {
                            Some(target)
                        })
                    }
                    DOMMutationType::kAppendChild
                    | DOMMutationType::kInsertBefore
                    | DOMMutationType::kRemoveChild => {
                        if let Some(child) = document.FindNodeById(mutation.child_node_id) {
                            add(document.Node(child).Parent());
                            if document.Node(child).Type() == DOMNodeType::kDocumentFragment {
                                add(Some(child));
                            }
                        }
                        add(Some(target));
                    }
                    _ => return None,
                }
            }
        }
        if records.is_empty() {
            return None;
        }
        let mut nodes = Vec::new();
        match mutation.mutation_type {
            DOMMutationType::kSetInnerHTML | DOMMutationType::kSetTextContent => {
                let contents = if document.Node(target).IsHTMLElement("template") {
                    document.TemplateContents(target).unwrap_or(target)
                } else {
                    target
                };
                nodes.extend(
                    document
                        .Node(contents)
                        .Children()
                        .iter()
                        .map(|&child| document.Node(child).Id()),
                );
            }
            DOMMutationType::kAppendChild
            | DOMMutationType::kInsertBefore
            | DOMMutationType::kRemoveChild => {
                if let Some(child) = document.FindNodeById(mutation.child_node_id) {
                    if document.Node(child).Type() == DOMNodeType::kDocumentFragment {
                        nodes.extend(
                            document
                                .Node(child)
                                .Children()
                                .iter()
                                .map(|&child| document.Node(child).Id()),
                        );
                    } else {
                        nodes.push(document.Node(child).Id());
                    }
                }
            }
            _ => {}
        }
        let roots = if nodes.is_empty() {
            None
        } else {
            let reason = self.allocate_collection_id();
            for &node in &nodes {
                self.host_graph.SetRoot(node, reason, true);
            }
            Some(PendingRoots {
                graph: self.host_graph.clone(),
                reason,
                nodes,
            })
        };
        Some(MutationNotification { records, roots })
    }

    pub fn DeliverMutationNotification(
        bindings: &Rc<RefCell<Self>>,
        notification: Option<MutationNotification>,
        call: &mut dyn FnMut(&JavaScriptFunction, &[HostValue]),
    ) {
        let Some(notification) = notification else {
            return;
        };
        let MutationNotification {
            records,
            roots: _roots,
        } = notification;
        for record in records {
            let invocation = {
                let bindings = bindings.borrow();
                let owner = bindings.document.borrow();
                let document = owner.GetDocument();
                let node = document.Node(record.node);
                let mut name = HostValue::Null(JavaScriptNull);
                let mut namespace = HostValue::Null(JavaScriptNull);
                let mut old = HostValue::Null(JavaScriptNull);
                let mut previous = HostValue::Null(JavaScriptNull);
                let mut next = HostValue::Null(JavaScriptNull);
                // Null asks RecordWithEmptyNodeLists to instantiate on read.
                let mut added = HostValue::Null(JavaScriptNull);
                let mut removed = HostValue::Null(JavaScriptNull);
                let kind = match record.data {
                    RecordData::Attributes {
                        name: attribute,
                        namespace: ns,
                        old_value,
                    } => {
                        name = HostValue::String(attribute);
                        if !ns.is_empty() {
                            namespace = HostValue::String(ns);
                        }
                        if let Some(value) = old_value {
                            old = HostValue::String(value);
                        }
                        "attributes"
                    }
                    RecordData::CharacterData { old_value } => {
                        if let Some(value) = old_value {
                            old = HostValue::String(value);
                        }
                        "characterData"
                    }
                    RecordData::ChildList { children } => {
                        let current = node
                            .Children()
                            .iter()
                            .map(|&child| document.Node(child).Id())
                            .collect::<Vec<_>>();
                        let removed_nodes = children
                            .iter()
                            .copied()
                            .filter(|id| !current.contains(id))
                            .collect::<Vec<_>>();
                        let added_nodes = current
                            .iter()
                            .copied()
                            .filter(|id| !children.contains(id))
                            .collect::<Vec<_>>();
                        if added_nodes.is_empty() && removed_nodes.is_empty() {
                            continue;
                        }
                        let (order, changed) = if added_nodes.is_empty() {
                            (&children, &removed_nodes)
                        } else {
                            (&current, &added_nodes)
                        };
                        let first = order.iter().position(|id| id == &changed[0]).unwrap();
                        let last = order
                            .iter()
                            .position(|id| id == changed.last().unwrap())
                            .unwrap();
                        if first != 0 {
                            previous = HostValue::Object(HostObjectRef {
                                id: order[first - 1],
                            });
                        }
                        if last + 1 < order.len() {
                            next = HostValue::Object(HostObjectRef {
                                id: order[last + 1],
                            });
                        }
                        added = HostValue::Object(bindings.StaticNodeList(added_nodes));
                        removed = HostValue::Object(bindings.StaticNodeList(removed_nodes));
                        "childList"
                    }
                };
                let recipients = HostValue::Record(
                    record
                        .recipients
                        .into_iter()
                        .map(|recipient| {
                            (
                                recipient.observer.0.to_string(),
                                HostValue::Boolean(recipient.old_value),
                            )
                        })
                        .collect(),
                );
                let args = [
                    HostValue::Object(HostObjectRef { id: node.Id() }),
                    HostValue::String(kind.into()),
                    name,
                    namespace,
                    old,
                    added,
                    removed,
                    previous,
                    next,
                    recipients,
                ];
                (bindings.method_wrappers["__mutationRecord"].clone(), args)
            };
            // No arena or host borrow survives entry into JavaScript.
            call(&invocation.0, &invocation.1);
        }
    }
}

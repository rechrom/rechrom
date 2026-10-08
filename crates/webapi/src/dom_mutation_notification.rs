use super::*;

struct Snapshot {
    node: usize,
    children: Vec<HostObjectId>,
    old_value: String,
    had_attribute: bool,
}

// A source notifier owns pre-mutation snapshots, while the actual persistent
// arena and collection identity remain owned by DOMJavaScriptBindings.
pub struct MutationNotification {
    mutation: DOMMutation,
    snapshots: Vec<Snapshot>,
}

impl DOMJavaScriptBindings {
    // cpp: webapi/dom_bindings.cc:1214-1247
    pub fn PrepareMutationNotification(
        &self,
        mutation: &DOMMutation,
    ) -> Option<MutationNotification> {
        // Blink builds a MutationObserverInterestGroup before allocating old
        // values, child snapshots or mutation records. Keep the same fast path:
        // most documents have no active observer and must pay nothing here.
        if !self.mutation_observers_active
            || !self.runtime_realm.IsValid()
            || !self.method_wrappers.contains_key("__mutationRecord")
        {
            return None;
        }
        let owner = self.document.borrow();
        let document = owner.GetDocument();
        let target = document.FindNodeById(mutation.target_node_id)?;
        let mut snapshots = Vec::<Snapshot>::new();
        let mut add = |index: Option<usize>| {
            let Some(index) = index else {
                return;
            };
            if snapshots.iter().any(|s| s.node == index) {
                return;
            }
            let node = document.Node(index);
            let attribute = node.FindAttribute(&mutation.name);
            snapshots.push(Snapshot {
                node: index,
                children: node
                    .Children()
                    .iter()
                    .map(|&i| document.Node(i).Id())
                    .collect(),
                old_value: if mutation.mutation_type == DOMMutationType::kSetTextContent {
                    node.Data().to_owned()
                } else {
                    attribute.map_or_else(String::new, |a| a.value.clone())
                },
                had_attribute: attribute.is_some(),
            });
        };
        match mutation.mutation_type {
            DOMMutationType::kSetAttribute
            | DOMMutationType::kRemoveAttribute
            | DOMMutationType::kSetTextContent => add(Some(target)),
            DOMMutationType::kSetInnerHTML => {
                add(if document.Node(target).IsHTMLElement("template") {
                    document.TemplateContents(target)
                } else {
                    Some(target)
                });
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
        Some(MutationNotification {
            mutation: mutation.clone(),
            snapshots,
        })
    }

    // cpp: webapi/dom_bindings.cc:1248-1276
    // Release both arena and host borrows before each Source __mutationRecord
    // call: matching observers performs live DOM contains/parent queries.
    pub fn DeliverMutationNotification(
        bindings: &Rc<RefCell<Self>>,
        notification: Option<MutationNotification>,
        call: &mut dyn FnMut(&JavaScriptFunction, &[HostValue]),
    ) {
        let Some(notification) = notification else {
            return;
        };
        for snapshot in notification.snapshots {
            let invocation = {
                let bindings = bindings.borrow();
                let owner = bindings.document.borrow();
                let document = owner.GetDocument();
                let node = document.Node(snapshot.node);
                let mutation = &notification.mutation;
                let mut name = HostValue::Null(JavaScriptNull);
                let mut old = HostValue::Null(JavaScriptNull);
                let mut previous = HostValue::Null(JavaScriptNull);
                let mut next = HostValue::Null(JavaScriptNull);
                let mut added = Vec::new();
                let mut removed = Vec::new();
                let kind;
                if matches!(
                    mutation.mutation_type,
                    DOMMutationType::kSetAttribute | DOMMutationType::kRemoveAttribute
                ) {
                    if mutation.mutation_type == DOMMutationType::kRemoveAttribute
                        && !snapshot.had_attribute
                    {
                        continue;
                    }
                    kind = "attributes";
                    name = HostValue::String(mutation.name.clone());
                    if snapshot.had_attribute {
                        old = HostValue::String(snapshot.old_value);
                    }
                } else if mutation.mutation_type == DOMMutationType::kSetTextContent
                    && matches!(
                        node.Type(),
                        DOMNodeType::kText
                            | DOMNodeType::kComment
                            | DOMNodeType::kProcessingInstruction
                    )
                {
                    kind = "characterData";
                    old = HostValue::String(snapshot.old_value);
                } else {
                    kind = "childList";
                    let current: Vec<_> = node
                        .Children()
                        .iter()
                        .map(|&i| document.Node(i).Id())
                        .collect();
                    removed.extend(
                        snapshot
                            .children
                            .iter()
                            .copied()
                            .filter(|id| !current.contains(id)),
                    );
                    added.extend(
                        current
                            .iter()
                            .copied()
                            .filter(|id| !snapshot.children.contains(id)),
                    );
                    if added.is_empty() && removed.is_empty() {
                        continue;
                    }
                    let (order, changed) = if added.is_empty() {
                        (&snapshot.children, &removed)
                    } else {
                        (&current, &added)
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
                }
                let list = |nodes: Vec<HostObjectId>| {
                    let id = bindings.allocate_collection_id();
                    bindings.collections.borrow_mut().insert(
                        id,
                        NodeCollection {
                            root: 0,
                            kind: "snapshot".into(),
                            query: String::new(),
                            snapshot: nodes,
                            membership_cache: RefCell::new(None),
                        },
                    );
                    HostValue::Object(HostObjectRef { id })
                };
                let args = [
                    HostValue::Object(HostObjectRef { id: node.Id() }),
                    HostValue::String(kind.into()),
                    name,
                    old,
                    list(added),
                    list(removed),
                    previous,
                    next,
                ];
                (bindings.method_wrappers["__mutationRecord"].clone(), args)
            };
            // Source ignores this helper's JavaScriptResult; observer callbacks
            // run through the existing JS microtask/timer exception boundaries.
            call(&invocation.0, &invocation.1);
        }
    }
}

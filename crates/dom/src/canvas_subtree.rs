//! Element canvas-subtree flag, Chromium element.cc:4213-4231,4423-4507;
//! Node flat-tree parent lifecycle cc:3804-3823. Queries read retained state.
use super::*;
impl PersistentDocument {
    pub fn IsInCanvasSubtree(&self, node: usize) -> bool {
        self.arena[node].is_in_canvas_subtree
    }
    pub fn IsCanvasOrInCanvasSubtree(&self, node: usize) -> bool {
        self.IsInCanvasSubtree(node) || self.arena[node].IsHTMLElement("canvas")
    }
    // Document::LocalOwner crosses the frame boundary as its immutable source
    // predicate. None means this Document has no local owner (e.g. main frame).
    pub fn SetDocumentCanvasOwnerState(&mut self, document: DOMOwnerHandle, owner: Option<bool>) {
        if let Some(value) = owner {
            self.canvas_document_owner_state.insert(document, value);
        } else {
            self.canvas_document_owner_state.remove(&document);
        }
        let root = self
            .FindNodeById(document.node_id)
            .filter(|&root| self.arena[root].node_type == DOMNodeType::kDocument)
            .expect("canonical owner Document");
        if let Some(element) = self.arena[root]
            .children
            .iter()
            .copied()
            .find(|&node| self.arena[node].node_type == DOMNodeType::kElement)
        {
            self.SetIsInCanvasSubtree(element, owner.unwrap_or(false));
        }
    }
    fn CanvasDocumentOwnerStateForRootElement(&self, node: usize) -> Option<bool> {
        if !self.IsConnectedToDocument(node) {
            return None;
        }
        let parent = self.arena[node].parent?;
        if self.arena[parent].node_type != DOMNodeType::kDocument
            || self.arena[parent]
                .children
                .iter()
                .copied()
                .find(|&child| self.arena[child].node_type == DOMNodeType::kElement)
                != Some(node)
        {
            return None;
        }
        self.canvas_document_owner_state
            .get(&self.NodeDocumentHandle(node))
            .copied()
    }
    fn ComputeIsInCanvasSubtree(&self, node: usize) -> bool {
        let mut parent = self.FlatTreeParentElement(node);
        while parent.is_some_and(|parent| self.IsSlotSupportingAssignment(parent)) {
            parent = self.FlatTreeParentElement(parent.unwrap());
        }
        if let Some(parent) = parent {
            return self.IsCanvasOrInCanvasSubtree(parent);
        }
        self.CanvasDocumentOwnerStateForRootElement(node)
            .unwrap_or(false)
    }
    pub(super) fn SetIsInCanvasSubtree(&mut self, node: usize, mut value: bool) {
        if self.arena[node].node_type != DOMNodeType::kElement
            || self.IsInCanvasSubtree(node) == value
        {
            return;
        }
        self.arena[node].is_in_canvas_subtree = value;
        self.editing_mutations
            .push(DocumentEditingMutation::ElementCanvasSubtreeChanged {
                node: self.arena[node].id,
                in_canvas: value,
            });
        if !value && self.arena[node].IsHTMLElement("canvas") {
            value = true;
        }
        let children = self.FlatTreeChildren(node).into_owned();
        for child in children {
            self.SetIsInCanvasSubtree(child, value);
        }
        let pseudos = self
            .generated_pseudo_elements
            .get(&node)
            .map(|pseudos| pseudos.values().copied().collect::<Vec<_>>())
            .unwrap_or_default();
        for pseudo in pseudos {
            self.SetIsInCanvasSubtree(pseudo, value);
        }
    }
    pub(super) fn CanvasSubtreeInsertedInto(&mut self, node: usize) {
        if self.arena[node].node_type != DOMNodeType::kElement {
            return;
        }
        let Some(parent) = self.ParentOrShadowHostElement(node) else {
            if self.CanvasDocumentOwnerStateForRootElement(node) == Some(true) {
                self.SetIsInCanvasSubtree(node, true);
            }
            return;
        };
        if self.IsCanvasOrInCanvasSubtree(parent)
            && !(self.ShadowRoot(parent).is_some() && self.arena[node].parent == Some(parent))
            && !(self.IsSlotSupportingAssignment(parent) && self.HasAssignedNodesNoRecalc(parent))
        {
            self.SetIsInCanvasSubtree(node, true);
        }
    }
    // The existing slot/child mutation owner calls this after assignment facts
    // change. Recompute only affected direct members; Set propagates changed
    // bits through active flat children, including nested canvas boundaries.
    pub(super) fn CanvasFlatTreeChanged(&mut self, node: usize) {
        if self.arena[node].node_type == DOMNodeType::kElement {
            let value = self.ComputeIsInCanvasSubtree(node);
            self.SetIsInCanvasSubtree(node, value);
        }
        let scope = self
            .ShadowRoot(node)
            .or_else(|| self.ContainingShadowRoot(node))
            .or_else(|| {
                self.arena[node]
                    .parent
                    .and_then(|parent| self.ShadowRoot(parent))
            });
        let mut candidates = self.arena[node].children.clone();
        if let Some(root) = scope {
            candidates.extend(self.arena[root].children.iter().copied());
            if let Some(host) = self.ShadowHost(root) {
                candidates.extend(self.arena[host].children.iter().copied());
            }
            // SlotAssignment owns this exact membership. Only affected scope's
            // slots are visited, without a DOM/shadow-tree discovery walk.
            let slots = self
                .slots_by_name
                .get(&root)
                .into_iter()
                .flat_map(|names| names.values())
                .flat_map(|nodes| nodes.iter().copied())
                .collect::<Vec<_>>();
            for slot in slots {
                candidates.extend(self.arena[slot].children.iter().copied());
                candidates.extend(self.AssignedNodes(slot));
            }
        }
        let mut seen = std::collections::HashSet::new();
        for child in candidates {
            if seen.insert(child) && self.arena[child].node_type == DOMNodeType::kElement {
                let value = self.ComputeIsInCanvasSubtree(child);
                self.SetIsInCanvasSubtree(child, value);
            }
        }
    }
    pub(super) fn CanvasSlotAttributeChanged(&mut self, node: usize, name: &str, namespace: &str) {
        if !self.AttributeMayChangeFlatTree(node, name, namespace) {
            return;
        }
        if name == "name" {
            let root = self
                .ContainingShadowRoot(node)
                .expect("assignable slot scope");
            if let Some(names) = self.slots_by_name.get_mut(&root) {
                // Remove this slot's previous name membership before reading
                // the newly committed UTF-16 attribute. No UTF-8 projection
                // participates in named-slot identity.
                names.retain(|_, nodes| {
                    nodes.remove(&node);
                    !nodes.is_empty()
                });
            }
            let new_name = self.arena[node]
                .AttributeValueUTF16("name")
                .unwrap_or_else(|| Rc::from([]));
            self.slots_by_name
                .entry(root)
                .or_default()
                .entry(new_name)
                .or_default()
                .insert(node);
        }
        self.CanvasFlatTreeChanged(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canvas_flag_follows_assigned_content_fallback_and_exact_slot_names() {
        let mut d = PersistentDocument::new();
        let outer = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        d.AppendChild(d.Root(), outer);
        let canvas = d.CreateElementDefault(DOMNamespace::kHTML, "canvas".into());
        d.AppendChild(outer, canvas);
        let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        d.AppendChild(canvas, host);
        let root = d.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let slot = d.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
        d.AppendChild(root, slot);
        let fallback = d.CreateElementDefault(DOMNamespace::kHTML, "span".into());
        d.AppendChild(slot, fallback);
        assert!(d.IsInCanvasSubtree(fallback));
        let input = d.CreateElementDefault(DOMNamespace::kHTML, "input".into());
        d.AppendChild(host, input);
        assert_eq!(d.AssignedSlot(input), Some(slot));
        assert!(d.IsInCanvasSubtree(input));
        assert!(!d.IsInCanvasSubtree(fallback));
        d.SetAttribute(
            input,
            DOMAttribute {
                local_name: "slot".into(),
                value: "unassigned".into(),
                ..Default::default()
            },
        );
        assert!(!d.IsInCanvasSubtree(input));
        assert!(d.IsInCanvasSubtree(fallback));
        let second = d.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
        d.AppendChild(root, second);
        d.SetAttributeUTF16(
            slot,
            DOMAttribute {
                local_name: "name".into(),
                ..Default::default()
            },
            &[0xd800],
        );
        d.SetAttributeUTF16(
            second,
            DOMAttribute {
                local_name: "name".into(),
                ..Default::default()
            },
            &[0xd801],
        );
        d.SetAttributeUTF16(
            input,
            DOMAttribute {
                local_name: "slot".into(),
                ..Default::default()
            },
            &[0xd801],
        );
        assert_eq!(d.AssignedSlot(input), Some(second));
        assert!(d.IsInCanvasSubtree(input));
        d.Remove(host);
        assert!(!d.IsInCanvasSubtree(host));
        assert!(!d.IsInCanvasSubtree(input));
        assert!(!d.IsInCanvasSubtree(canvas));
    }
}

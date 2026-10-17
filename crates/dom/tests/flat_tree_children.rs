//! Direct flat-tree ownership is distinct from recursively flattened bindings.
use dom::persistent_document::{
    DOMAttribute, DOMNamespace, DOMShadowRootMode, DOMSlotAssignmentMode,
};

fn element(document: &mut dom::Document, tag: &str, parent: usize) -> usize {
    let node = document.CreateElementDefault(DOMNamespace::kHTML, tag.into());
    document.AppendChild(parent, node);
    node
}
fn attribute(document: &mut dom::Document, node: usize, name: &str, value: &str) {
    document.SetAttribute(
        node,
        DOMAttribute {
            local_name: name.into(),
            value: value.into(),
            ..Default::default()
        },
    );
}

#[test]
fn direct_flat_children_preserve_slots_and_select_only_active_slotable_content() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
    let slot = element(d, "slot", root);
    let nested = element(d, "slot", slot);
    let fallback = d.CreateText("fallback".into());
    d.AppendChild(nested, fallback);
    let comment = d.CreateComment("inert fallback".into());
    d.AppendChild(slot, comment);
    assert_eq!(d.FlatTreeChildren(host).as_ref(), &[slot]);
    assert_eq!(d.FlatTreeChildren(slot).as_ref(), &[nested]);
    assert_eq!(d.FlatTreeChildren(nested).as_ref(), &[fallback]);
    assert_eq!(d.FlattenedAssignedNodes(slot), vec![fallback]);
    let light = element(d, "span", host);
    let unmatched = element(d, "span", host);
    attribute(d, unmatched, "slot", "missing");
    assert_eq!(d.FlatTreeChildren(slot).as_ref(), &[light]);
    assert_eq!(d.FlatTreeParentElement(light), Some(slot));
    assert_eq!(d.FlatTreeParentElement(nested), None);
    assert_eq!(d.FlatTreeParentElement(unmatched), None);
    d.Remove(light);
    assert_eq!(d.FlatTreeChildren(slot).as_ref(), &[nested]);
}

#[test]
fn flat_tree_mutations_revoke_geometry_independently_of_style_equality() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    // Geometry invalidation belongs to the active document. Detached shadow
    // construction is covered separately by the retained style pipeline test.
    d.AppendChild(d.Root(), host);
    d.StyleStateMut().impact = Default::default();
    let before = d.StyleState().measurement_geometry_revision;
    let root = d.AttachShadowRootWithSlotAssignment(
        host,
        DOMShadowRootMode::kClosed,
        DOMSlotAssignmentMode::kManual,
    );
    assert!(d.StyleState().impact.reattach);
    assert_ne!(d.StyleState().measurement_geometry_revision, before);
    let slot = element(d, "slot", root);
    let a = element(d, "span", host);
    let b = d.CreateText("text".into());
    d.AppendChild(host, b);
    d.AssignSlotNodes(slot, vec![a, b]);
    assert_eq!(d.FlatTreeChildren(slot).as_ref(), &[a, b]);
    let before = d.StyleState().measurement_geometry_revision;
    d.AssignSlotNodes(slot, vec![b, a]);
    assert_ne!(d.StyleState().measurement_geometry_revision, before);
    assert_eq!(d.FlatTreeChildren(slot).as_ref(), &[b, a]);
    let before = d.StyleState().measurement_geometry_revision;
    assert!(!d.AssignSlotNodes(slot, vec![b, a]));
    assert_eq!(d.StyleState().measurement_geometry_revision, before);
    attribute(d, slot, "name", "renamed");
    assert_ne!(d.StyleState().measurement_geometry_revision, before);
    let before = d.StyleState().measurement_geometry_revision;
    attribute(d, slot, "name", "renamed");
    assert_eq!(d.StyleState().measurement_geometry_revision, before);
    attribute(d, a, "slot", "named");
    assert_ne!(d.StyleState().measurement_geometry_revision, before);
    let before = d.StyleState().measurement_geometry_revision;
    attribute(d, a, "slot", "named");
    assert_eq!(d.StyleState().measurement_geometry_revision, before);
    d.RemoveAttributeDefault(slot, "name");
    assert_ne!(d.StyleState().measurement_geometry_revision, before);
}

#[test]
fn ua_flat_tree_remains_canonical_even_when_layout_uses_existing_control_adapters() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let light = element(d, "span", host);
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kUserAgent);
    let native = element(d, "div", root);
    assert_eq!(d.FlatTreeChildren(host).as_ref(), &[native]);
    assert_eq!(d.FlatTreeParentElement(light), None);
    assert_eq!(d.FlatTreeParentElement(native), Some(host));
}

use dom::persistent_document::{
    DOMAttribute, DOMNamespace, DOMShadowRootMode, DOMSlotAssignmentMode,
};
use dom::{DOMEventPath, Document};

fn element(document: &mut Document, parent: usize, tag: &str) -> usize {
    let node = document.CreateElementDefault(DOMNamespace::kHTML, tag.into());
    document.AppendChild(parent, node);
    node
}

fn attribute(document: &mut Document, node: usize, name: &str, value: &str) {
    document.SetAttribute(
        node,
        DOMAttribute {
            local_name: name.into(),
            value: value.into(),
            ..Default::default()
        },
    );
}

fn ids(document: &Document, nodes: &[usize]) -> Vec<u64> {
    nodes.iter().map(|&node| document.Node(node).Id()).collect()
}

fn invocations(path: &DOMEventPath) -> Vec<u64> {
    path.Entries()
        .iter()
        .map(|entry| entry.invocation_target)
        .collect()
}

fn targets(path: &DOMEventPath) -> Vec<u64> {
    path.Entries()
        .iter()
        .map(|entry| entry.shadow_adjusted_target)
        .collect()
}

#[test]
fn shadow_path_reaches_host_and_document_where_raw_parent_walk_stops() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let shadow = d.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
    let target = element(d, shadow, "span");
    // The pre-fix interaction path followed only Parent(): this witness
    // captures the failure without retaining that algorithm in production.
    let mut raw_parent_path = Vec::new();
    let mut current = Some(target);
    while let Some(node) = current {
        raw_parent_path.push(d.Node(node).Id());
        current = d.Node(node).Parent();
    }
    assert_eq!(raw_parent_path, ids(d, &[target, shadow]));
    let path = DOMEventPath::new(d, target, true);
    assert_eq!(
        invocations(&path),
        ids(d, &[target, shadow, host, document])
    );
    assert_ne!(raw_parent_path, invocations(&path));
    assert_eq!(targets(&path), ids(d, &[target, target, host, host]));
    assert_eq!(path.Target(), d.Node(target).Id());
    assert!(path.ReachesDocument());
    assert_eq!(path.DocumentTarget(), Some(d.Node(document).Id()));
    assert_eq!(path.PostDispatchTarget(), Some(d.Node(host).Id()));
    assert_eq!(path.ComposedPath(d.Node(host).Id()), invocations(&path));
}

#[test]
fn noncomposed_internal_target_stops_at_its_original_shadow_root() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let shadow = d.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
    let target = element(d, shadow, "span");
    let path = DOMEventPath::new(d, target, false);
    assert_eq!(invocations(&path), ids(d, &[target, shadow]));
    assert_eq!(targets(&path), ids(d, &[target, target]));
    assert!(!path.ReachesDocument());
    assert_eq!(path.DocumentTarget(), None);
    assert_eq!(path.PostDispatchTarget(), None);
    assert!(path.ComposedPath(d.Node(host).Id()).is_empty());
}

#[test]
fn closed_and_ua_roots_hide_internals_only_from_outside_observers() {
    for mode in [DOMShadowRootMode::kClosed, DOMShadowRootMode::kUserAgent] {
        let mut owner = dom::DOM::new();
        let d = owner.GetDocumentMut();
        let document = d.Root();
        let host = element(d, document, "div");
        let shadow = d.AttachShadowRoot(host, mode);
        let target = element(d, shadow, "span");
        let path = DOMEventPath::new(d, target, true);
        assert_eq!(path.ComposedPath(d.Node(target).Id()), invocations(&path));
        assert_eq!(path.ComposedPath(d.Node(shadow).Id()), invocations(&path));
        assert_eq!(
            path.ComposedPath(d.Node(host).Id()),
            ids(d, &[host, document])
        );
        assert_eq!(
            path.ComposedPath(d.Node(document).Id()),
            ids(d, &[host, document])
        );
        assert_eq!(targets(&path), ids(d, &[target, target, host, host]));
    }
}

#[test]
fn nested_roots_retarget_per_scope_and_filter_each_closed_boundary() {
    for outer_mode in [DOMShadowRootMode::kOpen, DOMShadowRootMode::kClosed] {
        for inner_mode in [DOMShadowRootMode::kOpen, DOMShadowRootMode::kClosed] {
            let mut owner = dom::DOM::new();
            let d = owner.GetDocumentMut();
            let document = d.Root();
            let outer_host = element(d, document, "div");
            let outer = d.AttachShadowRoot(outer_host, outer_mode);
            let inner_host = element(d, outer, "div");
            let inner = d.AttachShadowRoot(inner_host, inner_mode);
            let target = element(d, inner, "span");
            let path = DOMEventPath::new(d, target, true);
            let full = ids(d, &[target, inner, inner_host, outer, outer_host, document]);
            assert_eq!(invocations(&path), full);
            assert_eq!(
                targets(&path),
                ids(
                    d,
                    &[target, target, inner_host, inner_host, outer_host, outer_host]
                )
            );
            assert_eq!(path.ComposedPath(d.Node(target).Id()), full);
            let outer_view = if inner_mode == DOMShadowRootMode::kClosed {
                ids(d, &[inner_host, outer, outer_host, document])
            } else {
                full.clone()
            };
            assert_eq!(path.ComposedPath(d.Node(outer).Id()), outer_view);
            let document_view = if outer_mode == DOMShadowRootMode::kClosed {
                ids(d, &[outer_host, document])
            } else {
                outer_view
            };
            assert_eq!(path.ComposedPath(d.Node(document).Id()), document_view);
        }
    }
}

#[test]
fn named_slot_noncomposed_light_target_crosses_closed_root_without_retargeting() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let slot = element(d, root, "slot");
    attribute(d, slot, "name", "content");
    let target = element(d, host, "span");
    attribute(d, target, "slot", "content");
    assert_eq!(d.AssignedSlot(target), Some(slot));
    let path = DOMEventPath::new(d, target, false);
    assert_eq!(
        invocations(&path),
        ids(d, &[target, slot, root, host, document])
    );
    assert_eq!(targets(&path), vec![d.Node(target).Id(); 5]);
    assert_eq!(path.ComposedPath(d.Node(slot).Id()), invocations(&path));
    assert_eq!(
        path.ComposedPath(d.Node(target).Id()),
        ids(d, &[target, host, document])
    );
    assert_eq!(
        path.ComposedPath(d.Node(document).Id()),
        ids(d, &[target, host, document])
    );
    assert!(path.ReachesDocument());
    assert_eq!(path.PostDispatchTarget(), Some(d.Node(target).Id()));
}

#[test]
fn manual_slot_noncomposed_text_target_uses_retained_assignment() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let root = d.AttachShadowRootWithSlotAssignment(
        host,
        DOMShadowRootMode::kOpen,
        DOMSlotAssignmentMode::kManual,
    );
    let slot = element(d, root, "slot");
    let target = d.CreateText("assigned".into());
    d.AppendChild(host, target);
    assert_eq!(
        invocations(&DOMEventPath::new(d, target, false)),
        ids(d, &[target, host, document])
    );
    d.AssignSlotNodes(slot, vec![target]);
    let path = DOMEventPath::new(d, target, false);
    assert_eq!(
        invocations(&path),
        ids(d, &[target, slot, root, host, document])
    );
    assert_eq!(targets(&path), vec![d.Node(target).Id(); 5]);
    assert_eq!(path.ComposedPath(d.Node(document).Id()), invocations(&path));
}

#[test]
fn noncomposed_target_crosses_nested_slot_root_then_stops_at_its_own_root() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let outer_host = element(d, document, "div");
    let outer = d.AttachShadowRoot(outer_host, DOMShadowRootMode::kOpen);
    let host = element(d, outer, "div");
    let inner = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let slot = element(d, inner, "slot");
    let target = element(d, host, "span");
    let path = DOMEventPath::new(d, target, false);
    assert_eq!(
        invocations(&path),
        ids(d, &[target, slot, inner, host, outer])
    );
    assert_eq!(targets(&path), vec![d.Node(target).Id(); 5]);
    assert_eq!(
        path.ComposedPath(d.Node(outer).Id()),
        ids(d, &[target, host, outer])
    );
    assert!(!path.ReachesDocument());
    assert_eq!(path.PostDispatchTarget(), None);
}

#[test]
fn sibling_shadow_branches_filter_closed_target_scope_from_slot_observer() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let slot_root = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let slot = element(d, slot_root, "slot");
    let light_host = element(d, host, "div");
    let light_root = d.AttachShadowRoot(light_host, DOMShadowRootMode::kClosed);
    let target = element(d, light_root, "span");
    let path = DOMEventPath::new(d, target, true);
    assert_eq!(
        invocations(&path),
        ids(
            d,
            &[target, light_root, light_host, slot, slot_root, host, document]
        )
    );
    assert_eq!(
        targets(&path),
        ids(
            d,
            &[target, target, light_host, light_host, light_host, light_host, light_host]
        )
    );
    assert_eq!(
        path.ComposedPath(d.Node(slot).Id()),
        ids(d, &[light_host, slot, slot_root, host, document])
    );
    assert_eq!(
        path.ComposedPath(d.Node(target).Id()),
        ids(d, &[target, light_root, light_host, host, document])
    );
    assert_eq!(
        path.ComposedPath(d.Node(document).Id()),
        ids(d, &[light_host, host, document])
    );
}

#[test]
fn detached_fragment_keeps_identity_and_does_not_reach_window() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let fragment = d.CreateDocumentFragment();
    let parent = element(d, fragment, "div");
    let target = element(d, parent, "span");
    let path = DOMEventPath::new(d, target, true);
    assert_eq!(invocations(&path), ids(d, &[target, parent, fragment]));
    assert_eq!(targets(&path), vec![d.Node(target).Id(); 3]);
    assert_eq!(path.ComposedPath(d.Node(fragment).Id()), invocations(&path));
    assert!(!path.ReachesDocument());
    assert_eq!(path.DocumentTarget(), None);
    assert_eq!(path.PostDispatchTarget(), Some(d.Node(target).Id()));
}

#[test]
fn detached_shadow_host_and_secondary_document_do_not_supply_active_window() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let shadow = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let target = element(d, shadow, "span");
    let path = DOMEventPath::new(d, target, true);
    assert_eq!(invocations(&path), ids(d, &[target, shadow, host]));
    assert_eq!(path.ComposedPath(d.Node(host).Id()), ids(d, &[host]));
    assert!(!path.ReachesDocument());
    assert_eq!(path.PostDispatchTarget(), Some(d.Node(host).Id()));
    let secondary_document = d.CreateHTMLDocument(None);
    d.AppendChild(secondary_document, host);
    let secondary = DOMEventPath::new(d, target, true);
    assert_eq!(
        invocations(&secondary),
        ids(d, &[target, shadow, host, secondary_document])
    );
    assert!(!secondary.ReachesDocument());
    assert_eq!(secondary.DocumentTarget(), None);
    assert_eq!(secondary.PostDispatchTarget(), Some(d.Node(host).Id()));
}

#[test]
fn path_retains_assignment_scope_visibility_and_targets_across_mutation() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let root = d.AttachShadowRootWithSlotAssignment(
        host,
        DOMShadowRootMode::kClosed,
        DOMSlotAssignmentMode::kManual,
    );
    let slot = element(d, root, "slot");
    let target = element(d, host, "span");
    d.AssignSlotNodes(slot, vec![target]);
    let path = DOMEventPath::new(d, target, true);
    let original_entries = path.Entries().to_vec();
    let original_document_view = path.ComposedPath(d.Node(document).Id());
    let original_slot_view = path.ComposedPath(d.Node(slot).Id());
    // The observer itself leaves the closed root and the target is reassigned
    // to a newly attached root. Neither mutation may rewrite this dispatch.
    let new_host = element(d, document, "div");
    let new_root = d.AttachShadowRoot(new_host, DOMShadowRootMode::kOpen);
    let new_slot = element(d, new_root, "slot");
    d.AssignSlotNodes(slot, Vec::new());
    d.AppendChild(document, slot);
    d.AppendChild(new_host, target);
    d.Remove(host);
    assert_eq!(d.AssignedSlot(target), Some(new_slot));
    assert_eq!(path.Entries(), original_entries);
    assert_eq!(
        path.ComposedPath(d.Node(document).Id()),
        original_document_view
    );
    assert_eq!(path.ComposedPath(d.Node(slot).Id()), original_slot_view);
    assert_eq!(path.PostDispatchTarget(), Some(d.Node(target).Id()));
    assert!(path.ReachesDocument());
    assert_ne!(
        invocations(&DOMEventPath::new(d, target, true)),
        invocations(&path)
    );
    assert!(path.ComposedPath(d.Node(new_slot).Id()).is_empty());
}

#[test]
fn internal_shadow_target_snapshot_survives_reparenting_into_light_dom() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let host = element(d, document, "div");
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let target = element(d, root, "span");
    let confined = DOMEventPath::new(d, target, false);
    let composed = DOMEventPath::new(d, target, true);
    d.AppendChild(document, target);
    assert_eq!(confined.PostDispatchTarget(), None);
    assert_eq!(composed.PostDispatchTarget(), Some(d.Node(host).Id()));
    assert_eq!(
        composed.ComposedPath(d.Node(document).Id()),
        ids(d, &[host, document])
    );
    assert_eq!(targets(&composed), ids(d, &[target, target, host, host]));
}

#[test]
fn document_and_shadow_root_targets_preserve_target_first_contract() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let document_path = DOMEventPath::new(d, document, false);
    assert_eq!(invocations(&document_path), ids(d, &[document]));
    assert_eq!(targets(&document_path), ids(d, &[document]));
    assert!(document_path.ReachesDocument());
    let host = element(d, document, "div");
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
    let confined = DOMEventPath::new(d, root, false);
    assert_eq!(invocations(&confined), ids(d, &[root]));
    assert_eq!(confined.PostDispatchTarget(), None);
    let composed = DOMEventPath::new(d, root, true);
    assert_eq!(invocations(&composed), ids(d, &[root, host, document]));
    assert_eq!(targets(&composed), ids(d, &[root, host, host]));
}

#[test]
fn unknown_or_window_observer_has_no_dom_composed_path() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let document = d.Root();
    let target = element(d, document, "div");
    let other = element(d, document, "span");
    let path = DOMEventPath::new(d, target, true);
    assert!(path.ComposedPath(0).is_empty());
    assert!(path.ComposedPath(d.Node(other).Id()).is_empty());
    assert!(path.ComposedPath(u64::MAX).is_empty());
}

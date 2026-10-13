use dom::dom_mutation::{
    DOMMutation, DOMMutationType, DOMMutationTypeName, DOMMutationTypeNameRaw, TryApplyDOMMutations,
};
use dom::persistent_document::{DOMNamespace, DOMShadowRootMode, DOMSlotAssignmentMode};

#[test]
fn slot_mutation_validates_entire_sequence_before_changing_native_owner() {
    let mut dom = dom::DOM::new();
    let d = dom.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let root = d.AttachShadowRootWithSlotAssignment(
        host,
        DOMShadowRootMode::kClosed,
        DOMSlotAssignmentMode::kManual,
    );
    let slot = d.CreateElementDefault(DOMNamespace::kHTML, "slot".into());
    let first = d.CreateText("first".into());
    let second = d.CreateElementDefault(DOMNamespace::kHTML, "span".into());
    let comment = d.CreateComment("not slotable".into());
    d.AppendChild(root, slot);
    d.AppendChild(host, first);
    d.AppendChild(host, second);
    let mutation = DOMMutation {
        mutation_type: DOMMutationType::kAssignSlotNodes,
        target_node_id: d.Node(slot).Id(),
        node_ids: vec![d.Node(first).Id()],
        ..Default::default()
    };
    TryApplyDOMMutations(d, &[mutation.clone()]).unwrap();
    for bad in [
        DOMMutation {
            target_node_id: d.Node(host).Id(),
            ..mutation.clone()
        },
        DOMMutation {
            node_ids: vec![d.Node(second).Id(), 0],
            ..mutation.clone()
        },
        DOMMutation {
            node_ids: vec![d.Node(second).Id(), d.Node(comment).Id()],
            ..mutation.clone()
        },
    ] {
        assert!(TryApplyDOMMutations(d, &[bad]).is_err());
        assert_eq!(d.AssignedNodes(slot), vec![first]);
        assert_eq!(d.ManuallyAssignedSlot(second), None);
    }
    TryApplyDOMMutations(
        d,
        &[DOMMutation {
            node_ids: vec![d.Node(second).Id(), d.Node(first).Id(), d.Node(second).Id()],
            ..mutation.clone()
        }],
    )
    .unwrap();
    assert_eq!(d.AssignedNodes(slot), vec![second, first]);
    assert_eq!(d.FlatTreeParentElement(first), Some(slot));
    assert_eq!(d.AssignedNodes(host), Vec::<usize>::new());
}

#[test]
fn raw_mutation_trace_names_track_actual_discriminants() {
    for kind in [
        DOMMutationType::kImportNode,
        DOMMutationType::kAppendChild,
        DOMMutationType::kSetAttribute,
        DOMMutationType::kExitFullscreen,
        DOMMutationType::kAssignSlotNodes,
    ] {
        assert_eq!(
            DOMMutationTypeNameRaw(kind as u8),
            DOMMutationTypeName(kind)
        );
    }
    assert_eq!(DOMMutationTypeNameRaw(u8::MAX), "unknown");
}

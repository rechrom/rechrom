use dom::dom_mutation::{DOMMutation, DOMMutationType, TryApplyDOMMutations};
use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMNodeType, DOMShadowRootMode};

#[test]
fn import_control_clones_live_state_and_destination_owned_ua_tree() {
    let mut dom = dom::DOM::new();
    let d = dom.GetDocumentMut();
    let input = d.CreateElement(
        DOMNamespace::kHTML,
        "input".into(),
        vec![DOMAttribute {
            local_name: "type".into(),
            value: "search".into(),
            ..Default::default()
        }],
    );
    d.SetControlValue(input, "typed".into());
    let dest = d.CreateHTMLDocument(None);
    let imported = d.ImportNode(input, dest, false);
    assert!(d.Node(imported).Parent().is_none());
    assert_eq!(d.ControlValue(imported), "typed");
    let owner = d.Node(imported).OwnerDocumentNode().unwrap();
    assert_eq!(owner.Id(), d.Node(dest).Id());
    let root = d.ShadowRoot(imported).expect("regenerated UA shadow tree");
    let mut nodes = vec![root];
    while let Some(n) = nodes.pop() {
        assert_eq!(d.Node(n).OwnerDocumentNode(), Some(owner));
        nodes.extend_from_slice(d.Node(n).Children());
    }
    assert_ne!(d.Node(input).OwnerDocumentNode(), Some(owner));
    let clone = d.CloneNode(imported, true);
    assert_eq!(d.Node(clone).OwnerDocumentNode(), Some(owner));
    assert_eq!(
        d.Node(d.ShadowRoot(clone).unwrap()).OwnerDocumentNode(),
        Some(owner)
    );
}

#[test]
fn import_mutation_validates_source_destination_and_expected_identity_before_allocation() {
    let mut dom = dom::DOM::new();
    let d = dom.GetDocumentMut();
    let source = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let shadow = d.AttachShadowRoot(source, DOMShadowRootMode::kOpen);
    let root = d.Root();
    let valid = DOMMutation {
        mutation_type: DOMMutationType::kImportNode,
        target_node_id: d.Node(root).Id(),
        source_node_id: d.Node(source).Id(),
        child_node_id: d.NextNodeId(),
        bool_value: true,
        ..Default::default()
    };
    let before = d.NodeCount();
    for bad in [
        DOMMutation {
            source_node_id: 0,
            ..valid.clone()
        },
        DOMMutation {
            source_node_id: d.Node(root).Id(),
            ..valid.clone()
        },
        DOMMutation {
            source_node_id: d.Node(shadow).Id(),
            ..valid.clone()
        },
        DOMMutation {
            target_node_id: d.Node(source).Id(),
            ..valid.clone()
        },
        DOMMutation {
            child_node_id: 0,
            ..valid.clone()
        },
    ] {
        assert!(TryApplyDOMMutations(d, &[bad]).is_err());
        assert_eq!(d.NodeCount(), before);
    }
    TryApplyDOMMutations(d, &[valid.clone()]).unwrap();
    let imported = d.FindNodeById(valid.child_node_id).unwrap();
    assert_eq!(d.Node(imported).Type(), DOMNodeType::kElement);
    assert!(d.Node(imported).Parent().is_none());
    assert!(
        d.ShadowRoot(imported).is_none(),
        "non-clonable author root is not copied"
    );
}

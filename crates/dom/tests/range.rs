use dom::persistent_document::DOMNamespace;
use dom::{DOMBoundaryPoint, DOMRangeError, DOM};

fn point(node_id: u64, offset: usize) -> DOMBoundaryPoint {
    DOMBoundaryPoint { node_id, offset }
}

#[test]
fn live_boundaries_follow_child_identity_and_removal() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    document.AppendChild(document.Root(), parent);
    let first = document.CreateText("a".into());
    let second = document.CreateElementDefault(DOMNamespace::kHTML, "span".into());
    let nested = document.CreateText("xyz".into());
    document.AppendChild(parent, first);
    document.AppendChild(parent, second);
    document.AppendChild(second, nested);
    let parent_id = document.Node(parent).Id();
    let range = document
        .CreateRange(point(parent_id, 1), point(document.Node(nested).Id(), 2))
        .unwrap();
    let snapshot = document.Range(&range).unwrap();
    let inserted = document.CreateText("before".into());
    document.InsertBefore(parent, inserted, first);
    assert_eq!(document.Range(&range).unwrap().start, point(parent_id, 2));
    assert_eq!(snapshot.start, point(parent_id, 1));
    document.Remove(first);
    assert_eq!(document.Range(&range).unwrap().start, point(parent_id, 1));
    document.Remove(second);
    let current = document.Range(&range).unwrap();
    assert_eq!(current.end, point(parent_id, 1));
    assert!(current.Collapsed());
}

#[test]
fn range_setters_validate_utf16_offsets_and_collapse_in_tree_order() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let parent = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let text = document.CreateText("a😀b".into());
    document.AppendChild(parent, text);
    let parent_id = document.Node(parent).Id();
    let text_id = document.Node(text).Id();
    let range = document
        .CreateRange(point(parent_id, 0), point(text_id, 4))
        .unwrap();
    assert!(!document.Range(&range).unwrap().Collapsed());
    assert_eq!(
        document.SetRangeEnd(&range, point(text_id, 5)),
        Err(DOMRangeError::InvalidOffset)
    );
    assert_eq!(document.Range(&range).unwrap().end.offset, 4);
    document.SetRangeStart(&range, point(parent_id, 1)).unwrap();
    assert!(document.Range(&range).unwrap().Collapsed());
    document
        .SetRange(&range, point(text_id, 1), point(text_id, 3))
        .unwrap();
    document.SetTextContent(text, "replacement".into());
    let current = document.Range(&range).unwrap();
    assert_eq!(current.start, point(text_id, 0));
    assert!(current.Collapsed());
    let detached = document.CreateText("other".into());
    document
        .SetRangeEnd(&range, point(document.Node(detached).Id(), 2))
        .unwrap();
    assert!(document.Range(&range).unwrap().Collapsed());
    let other_owner = DOM::new();
    assert!(other_owner.GetDocument().Range(&range).is_none());
}

#[test]
fn partial_ancestor_deletion_preserves_unselected_dom_and_collapses_between_sides() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let root = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let left = document.CreateElementDefault(DOMNamespace::kHTML, "b".into());
    let right = document.CreateElementDefault(DOMNamespace::kHTML, "i".into());
    let middle = document.CreateElementDefault(DOMNamespace::kHTML, "hr".into());
    let left_text = document.CreateText("abcd".into());
    let left_tail = document.CreateText("remove".into());
    let right_head = document.CreateText("remove".into());
    let right_text = document.CreateText("mnop".into());
    document.AppendChild(root, left);
    document.AppendChild(root, middle);
    document.AppendChild(root, right);
    document.AppendChild(left, left_text);
    document.AppendChild(left, left_tail);
    document.AppendChild(right, right_head);
    document.AppendChild(right, right_text);
    let range = document
        .CreateRange(
            point(document.Node(left_text).Id(), 2),
            point(document.Node(right_text).Id(), 2),
        )
        .unwrap();
    dom::dom_mutation::TryApplyDOMMutations(
        document,
        &[dom::dom_mutation::DOMMutation {
            mutation_type: dom::dom_mutation::DOMMutationType::kDeleteRangeContents,
            target_node_id: document.Node(root).Id(),
            range: Some(range.clone()),
            ..Default::default()
        }],
    )
    .unwrap();
    assert_eq!(document.Node(root).Children(), &[left, right]);
    assert_eq!(document.Node(left).Children(), &[left_text]);
    assert_eq!(document.Node(right).Children(), &[right_text]);
    assert_eq!(document.Node(left_text).Data(), "ab");
    assert_eq!(document.Node(right_text).Data(), "op");
    let current = document.Range(&range).unwrap();
    assert!(current.Collapsed());
    assert_eq!(current.start, point(document.Node(root).Id(), 1));
}

#[test]
fn range_insertion_splits_text_and_moves_other_live_boundaries() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let root = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let text = document.CreateText("abcd".into());
    document.AppendChild(root, text);
    let text_id = document.Node(text).Id();
    let root_id = document.Node(root).Id();
    let range = document
        .CreateRange(point(text_id, 2), point(text_id, 2))
        .unwrap();
    let watcher = document
        .CreateRange(point(text_id, 3), point(root_id, 1))
        .unwrap();
    let inserted = document.CreateText("X".into());
    document
        .InsertRangeNode(&range, document.Node(inserted).Id())
        .unwrap();
    let children = document.Node(root).Children();
    assert_eq!(children.len(), 3);
    let next = children[2];
    assert_eq!(document.Node(text).Data(), "ab");
    assert_eq!(document.Node(next).Data(), "cd");
    assert_eq!(
        document.Range(&watcher).unwrap().start,
        point(document.Node(next).Id(), 1)
    );
    assert_eq!(document.Range(&watcher).unwrap().end, point(root_id, 3));
    let current = document.Range(&range).unwrap();
    assert_eq!(current.start, point(text_id, 2));
    assert_eq!(current.end, point(root_id, 2));
    document.ReplaceData(next, 0, 1, "YZ").unwrap();
    assert_eq!(
        document.Range(&watcher).unwrap().start,
        point(document.Node(next).Id(), 0)
    );
    assert_eq!(document.Node(next).Data(), "YZd");
}

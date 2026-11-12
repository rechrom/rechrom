use dom::dom_mutation::{ApplyDOMMutations, DOMMutation, DOMMutationType};
use dom::persistent_document::{DOMAttribute, DOMNamespace};
use dom::DOM;

#[test]
fn text_control_editor_owns_live_text_and_value_sync_preserves_identity() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let input = document.CreateElement(
        DOMNamespace::kHTML,
        "input".into(),
        vec![DOMAttribute {
            local_name: "value".into(),
            value: "before".into(),
            ..Default::default()
        }],
    );
    let text = document.TextControlInnerEditorTextNode(input).unwrap();
    let start = document.TextControlInnerEditorPosition(input, 1).unwrap();
    let end = document.TextControlInnerEditorPosition(input, 3).unwrap();
    let range = document.CreateRange(start, end).unwrap();
    document.SetControlValue(input, "before".into());
    assert_eq!(document.TextControlInnerEditorTextNode(input), Some(text));
    assert_eq!(document.Range(&range).unwrap().start, start);
    document.SetTextContent(text, "after".into());
    ApplyDOMMutations(
        document,
        &[DOMMutation {
            mutation_type: DOMMutationType::kSyncTextControlValueFromInnerEditor,
            target_node_id: document.Node(input).Id(),
            ..Default::default()
        }],
    );
    assert_eq!(document.ControlValue(input), "after");
    assert_eq!(document.TextControlInnerEditorTextNode(input), Some(text));
    assert_eq!(document.TextControlInnerEditorValue(input), "after");
}

#[test]
fn textarea_editor_maps_utf16_offsets_over_real_text_and_break_nodes() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let textarea = document.CreateElementDefault(DOMNamespace::kHTML, "textarea".into());
    document.AppendText(textarea, "a😀\nb\n");
    assert_eq!(document.TextControlInnerEditorValue(textarea), "a😀\nb\n");
    let editor = document.TextControlInnerEditorElement(textarea).unwrap();
    let children = document.Node(editor).Children().to_vec();
    assert_eq!(children.len(), 5);
    assert_eq!(document.Node(children[0]).Data(), "a😀");
    assert!(document.Node(children[1]).IsHTMLElement("br"));
    assert_eq!(document.Node(children[2]).Data(), "b");
    assert!(document.Node(children[3]).IsHTMLElement("br"));
    assert!(document.TextControlIsPlaceholderBreak(children[4]));
    for offset in 0..=6 {
        let position = document
            .TextControlInnerEditorPosition(textarea, offset)
            .unwrap();
        assert_eq!(
            document.TextControlInnerEditorOffset(textarea, position),
            Some(offset)
        );
    }
    document.SetControlValue(textarea, "dirty".into());
    let text = document.TextControlInnerEditorTextNode(textarea).unwrap();
    document.SetTextContent(children[0], "detached".into());
    document.SetTextContent(textarea, "new default".into());
    assert_eq!(document.ControlValue(textarea), "dirty");
    assert_eq!(
        document.TextControlInnerEditorTextNode(textarea),
        Some(text)
    );
    assert_eq!(document.TextControlInnerEditorValue(textarea), "dirty");
}

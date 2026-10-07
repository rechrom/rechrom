use dom::persistent_document::{DOMAttribute, DOMNamespace};
use dom::DOM;

// cpp: dom/document_test.cc:8-45
#[test]
fn persistent_arena_reparenting_and_text_coalescing() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let html = document.CreateElementDefault(DOMNamespace::kHTML, "html".into());
    let body = document.CreateElementDefault(DOMNamespace::kHTML, "body".into());
    let table = document.CreateElementDefault(DOMNamespace::kHTML, "table".into());
    let text = document.CreateText("foster-parented".into());
    document.AppendChild(document.Root(), html);
    document.AppendChild(html, body);
    document.AppendChild(body, table);
    document.AppendChild(table, text);

    document.SetAttribute(
        table,
        DOMAttribute {
            local_name: "role".into(),
            value: "presentation".into(),
            ..Default::default()
        },
    );
    document.SetAttribute(
        table,
        DOMAttribute {
            local_name: "role".into(),
            value: "grid".into(),
            ..Default::default()
        },
    );
    assert_eq!(
        document.Node(table).FindAttribute("role").unwrap().value,
        "grid"
    );

    document.InsertBefore(body, text, table);
    assert_eq!(document.Node(body).Children(), &[text, table]);
    assert_eq!(document.Node(text).Parent(), Some(body));

    let formatting = document.CreateElementDefault(DOMNamespace::kHTML, "b".into());
    document.TakeAllChildren(body, formatting);
    document.AppendText(formatting, " one");
    document.AppendText(formatting, " two");
    document.AppendChild(body, formatting);
    assert_eq!(document.Node(formatting).Children().len(), 3);
    let last = *document.Node(formatting).Children().last().unwrap();
    assert_eq!(document.Node(last).Data(), " one two");
    assert_eq!(document.NodeCount(), 7);
}

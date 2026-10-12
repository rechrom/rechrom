use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};
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

#[test]
fn constructed_stylesheet_adoption_has_explicit_document_and_shadow_scope_owners() {
    let mut owner = DOM::new();
    let document = owner.GetDocumentMut();
    let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
    document.AppendChild(document.Root(), host);
    let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);

    let unattached = style::ParseCSS(".unattached{color:black}");
    assert_eq!(document.StyleSheetTreeScope(&unattached), None);

    let constructed = style::ParseCSS("@scope{.item{color:red}}");
    let identity = constructed.Identity();
    assert_eq!(
        document.RegisterConstructedStyleSheet(constructed).unwrap(),
        identity
    );
    assert_eq!(
        document.ConstructedStyleSheetOwnerDocument(identity),
        Some(document.RootHandle())
    );
    assert!(document
        .ActiveStyleSheetsForTreeScope(shadow)
        .next()
        .is_none());
    assert!(document.StyleSheetsForNode(document.Root()).is_empty());

    assert!(document
        .SetAdoptedStyleSheets(shadow, vec![identity, identity])
        .unwrap());
    assert!(document.IsStyleSheetAdoptedByTreeScope(identity, shadow));
    assert!(!document.IsStyleSheetAdoptedByTreeScope(identity, document.Root()));
    assert_eq!(document.AdoptedTreeScopesForStyleSheet(identity), [shadow]);
    assert_eq!(document.ShadowHost(shadow), Some(host));
    assert_eq!(
        document.AdoptedStyleSheetImplicitScopeTrigger(identity, shadow),
        Some(host)
    );
    assert_eq!(
        document
            .ActiveStyleSheetsForTreeScope(shadow)
            .map(|sheet| sheet.Identity())
            .collect::<Vec<_>>(),
        [identity, identity]
    );
    assert!(document
        .SetAdoptedStyleSheets(document.Root(), vec![identity])
        .unwrap());
    assert_eq!(
        document.AdoptedStyleSheetImplicitScopeTrigger(identity, document.Root()),
        None
    );
    assert_eq!(
        document.AdoptedTreeScopesForStyleSheet(identity),
        [document.Root(), shadow]
    );
    assert_eq!(
        document
            .ActiveStyleSheetsForTreeScope(document.Root())
            .map(|sheet| sheet.Identity())
            .collect::<Vec<_>>(),
        [identity]
    );

    let ordinary = style::ParseCSS(".ordinary{color:blue}");
    let ordinary_identity = ordinary.Identity();
    document.AppendStyleSheet(ordinary);
    assert_eq!(
        document
            .ActiveStyleSheetsForTreeScope(document.Root())
            .map(|sheet| sheet.Identity())
            .collect::<Vec<_>>(),
        [ordinary_identity, identity]
    );
    assert_eq!(
        document
            .ActiveStyleSheetsWithTreeScopes()
            .into_iter()
            .map(|(scope, sheet)| (scope, sheet.Identity()))
            .collect::<Vec<_>>(),
        [
            (document.Root(), ordinary_identity),
            (document.Root(), identity),
            (shadow, identity),
            (shadow, identity),
        ]
    );

    let mut other = DOM::new();
    let other_root = other.GetDocument().Root();
    assert_eq!(
        other
            .GetDocumentMut()
            .SetAdoptedStyleSheets(other_root, vec![identity]),
        Err(cssom::CSSOMAdoptionError::UnknownConstructedStyleSheet)
    );
}

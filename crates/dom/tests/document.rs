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

mod active_style_sheet_order {
    use super::*;
    use dom::persistent_document::PersistentDocument;

    fn append_owned_sheet(document: &mut PersistentDocument, node: usize) -> u64 {
        let mut sheet = style::ParseCSS(".item { color: red }");
        sheet.owner_node_id = document.Node(node).Id();
        let identity = sheet.Identity();
        document.AppendStyleSheet(sheet);
        identity
    }

    fn identities(document: &PersistentDocument, scope: usize) -> Vec<u64> {
        document
            .ActiveStyleSheetsForTreeScope(scope)
            .map(|sheet| sheet.Identity())
            .collect()
    }

    #[test]
    fn style_media_and_type_changes_dirty_the_parsed_owner() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let style_node = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let other_node = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        document.AppendChild(document.Root(), style_node);
        document.AppendChild(document.Root(), other_node);
        let owner_id = document.Node(style_node).Id();
        append_owned_sheet(document, style_node);

        for (name, initial, replacement) in [
            ("media", "(min-width: 700px)", "all"),
            ("type", "text/plain", "text/css"),
        ] {
            let attribute = DOMAttribute {
                local_name: name.into(),
                value: initial.into(),
                ..Default::default()
            };
            document.SetAttribute(style_node, attribute.clone());
            assert!(document.StyleState().dirty_style_elements.contains(&owner_id));
            append_owned_sheet(document, style_node);
            assert!(document.StyleState().parsed_style_elements.contains(&owner_id));
            assert!(!document.StyleState().dirty_style_elements.contains(&owner_id));

            // Equal writes and attributes outside the HTML owner contract do
            // not trigger another stylesheet parse.
            document.SetAttribute(style_node, attribute.clone());
            document.SetAttribute(other_node, attribute.clone());
            let mut namespaced = attribute.clone();
            namespaced.namespace_uri = "urn:test".into();
            document.SetAttribute(style_node, namespaced);
            document.RemoveAttribute(style_node, name, "urn:test");
            assert!(!document.StyleState().dirty_style_elements.contains(&owner_id));

            document.SetAttribute(
                style_node,
                DOMAttribute {
                    value: replacement.into(),
                    ..attribute
                },
            );
            assert!(document.StyleState().dirty_style_elements.contains(&owner_id));
            append_owned_sheet(document, style_node);
            document.RemoveAttributeDefault(style_node, name);
            assert!(document.StyleState().dirty_style_elements.contains(&owner_id));
            append_owned_sheet(document, style_node);
            document.RemoveAttributeDefault(style_node, name);
            assert!(!document.StyleState().dirty_style_elements.contains(&owner_id));
        }
    }

    #[test]
    fn candidates_follow_dom_order_after_completion_subtree_moves_and_reconnection() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let root = document.Root();
        let first_group = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let last_group = document.CreateElementDefault(DOMNamespace::kHTML, "div".into());
        let first = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let link = document.CreateElementDefault(DOMNamespace::kHTML, "link".into());
        let last = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        document.AppendChild(root, first_group);
        document.AppendChild(first_group, first);
        document.AppendChild(root, link);
        document.AppendChild(root, last_group);
        document.AppendChild(last_group, last);

        // Resource completion order must not become cascade source order.
        let last_id = append_owned_sheet(document, last);
        let link_id = append_owned_sheet(document, link);
        let first_id = append_owned_sheet(document, first);
        assert_eq!(identities(document, root), [first_id, link_id, last_id]);

        let revision = document.StyleState().sheet_revision;
        document.InsertBefore(root, last_group, first_group);
        assert!(document.StyleState().sheet_revision > revision);
        assert_eq!(identities(document, root), [last_id, first_id, link_id]);

        document.InsertBefore(first_group, link, first);
        assert_eq!(identities(document, root), [last_id, link_id, first_id]);
        document.Remove(last_group);
        assert_eq!(identities(document, root), [link_id, first_id]);
        document.AppendChild(root, last_group);
        assert_eq!(identities(document, root), [link_id, first_id, last_id]);
    }

    #[test]
    fn ownerless_prefix_and_adopted_suffix_keep_order_and_multiplicity() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let root = document.Root();
        let first = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let last = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        document.AppendChild(root, first);
        document.AppendChild(root, last);

        let last_id = append_owned_sheet(document, last);
        let injected_first = style::ParseCSS(".item { color: blue }");
        let injected_first_id = injected_first.Identity();
        document.AppendStyleSheet(injected_first);
        let first_id = append_owned_sheet(document, first);
        let injected_last = style::ParseCSS(".item { color: green }");
        let injected_last_id = injected_last.Identity();
        document.AppendStyleSheet(injected_last);
        let adopted_first = document
            .RegisterConstructedStyleSheet(style::ParseCSS(".item { color: purple }"))
            .unwrap();
        let adopted_last = document
            .RegisterConstructedStyleSheet(style::ParseCSS(".item { color: orange }"))
            .unwrap();
        document
            .SetAdoptedStyleSheets(root, vec![adopted_last, adopted_first, adopted_last])
            .unwrap();
        assert_eq!(
            identities(document, root),
            [
                injected_first_id,
                injected_last_id,
                first_id,
                last_id,
                adopted_last,
                adopted_first,
                adopted_last,
            ]
        );
        document.InsertBefore(root, last, first);
        assert_eq!(
            identities(document, root),
            [
                injected_first_id,
                injected_last_id,
                last_id,
                first_id,
                adopted_last,
                adopted_first,
                adopted_last,
            ]
        );
    }

    #[test]
    fn shadow_candidates_use_their_own_tree_order_after_scope_moves() {
        let mut owner = DOM::new();
        let document = owner.GetDocumentMut();
        let root = document.Root();
        let host = document.CreateElementDefault(DOMNamespace::kHTML, "x-host".into());
        document.AppendChild(root, host);
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let first = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let last = document.CreateElementDefault(DOMNamespace::kHTML, "link".into());
        document.AppendChild(shadow, first);
        document.AppendChild(shadow, last);
        let last_id = append_owned_sheet(document, last);
        let first_id = append_owned_sheet(document, first);
        let adopted = document
            .RegisterConstructedStyleSheet(style::ParseCSS(".item { color: blue }"))
            .unwrap();
        document.SetAdoptedStyleSheets(shadow, vec![adopted]).unwrap();
        assert_eq!(identities(document, shadow), [first_id, last_id, adopted]);
        assert!(identities(document, root).is_empty());

        document.InsertBefore(shadow, last, first);
        assert_eq!(identities(document, shadow), [last_id, first_id, adopted]);
        document.AppendChild(root, first);
        assert_eq!(identities(document, shadow), [last_id, adopted]);
        assert_eq!(identities(document, root), [first_id]);
        document.Remove(host);
        assert_eq!(
            document
                .ActiveStyleSheetsWithTreeScopes()
                .into_iter()
                .map(|(scope, sheet)| (scope, sheet.Identity()))
                .collect::<Vec<_>>(),
            [(root, first_id)]
        );
    }
}

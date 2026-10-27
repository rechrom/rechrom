use dom::persistent_document::{DOMAttribute, DOMNamespace};
use dom::Document;
use style::persistent_selector::PersistentSelectorService;
fn element(d: &mut Document, parent: usize, tag: &str, id: &str, class: &str) -> usize {
    let i = d.CreateElementDefault(DOMNamespace::kHTML, tag.into());
    for (name, value) in [("id", id), ("class", class)] {
        if !value.is_empty() {
            d.SetAttribute(
                i,
                DOMAttribute {
                    local_name: name.into(),
                    value: value.into(),
                    ..Default::default()
                },
            );
        }
    }
    d.AppendChild(parent, i);
    i
}
#[test]
fn mapped_queries_read_real_dom_order_scope_and_mutations() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let html = element(d, d.Root(), "html", "", "");
    let root = element(d, html, "main", "root", "");
    let a = element(d, root, "p", "duplicate", "one two");
    d.AppendText(root, "between");
    let b = element(d, root, "p", "duplicate", "two");
    let section = element(d, root, "section", "section", "");
    let leaf = element(d, section, "span", "leaf", "leaf");
    let mut service = PersistentSelectorService::default();
    assert_eq!(
        service.QueryFirst(d, d.Root(), "#duplicate").unwrap(),
        Some(a)
    );
    assert_eq!(service.QueryAll(d, root, "#duplicate").unwrap(), vec![a, b]);
    assert_eq!(service.QueryFirst(d, root, "#root").unwrap(), None);
    assert_eq!(service.QueryAll(d, root, ":scope > p").unwrap(), vec![a, b]);
    assert_eq!(service.QueryAll(d, root, "& > p").unwrap(), vec![a, b]);
    assert!(service.Matches(d, a, "&").unwrap());
    assert!(service.Matches(d, a, ":is(p, !&)").unwrap());
    assert!(service
        .Matches(d, a, "main > p.one.two:first-child")
        .unwrap());
    assert!(service.Matches(d, b, "p + p:nth-child(2)").unwrap());
    assert_eq!(service.Closest(d, leaf, "main").unwrap(), Some(root));
    assert!(service
        .Matches(d, a, ":is(#duplicate,.absent):not(.absent):where(p)")
        .unwrap());
    assert!(service.Matches(d, root, ":has(> section)").unwrap());
    assert!(!service.Matches(d, root, ":has(> .leaf)").unwrap());
    assert!(service.Matches(d, root, ":has(> section .leaf)").unwrap());
    assert!(service.Matches(d, a, "*").unwrap());
    assert!(!service.Matches(d, a, "p::before").unwrap());
    d.SetAttribute(
        a,
        DOMAttribute {
            local_name: "class".into(),
            value: "changed".into(),
            ..Default::default()
        },
    );
    d.Remove(a);
    d.AppendChild(root, a);
    assert_eq!(service.QueryAll(d, root, "p").unwrap(), vec![b, a]);
    assert!(!service.Matches(d, a, ".one").unwrap());
    assert!(service.Matches(d, a, ".changed:last-child").unwrap());
    assert!(service.QueryFirst(d, root, "p >").is_err());
    assert!(service.QueryAll(d, root, "").is_err());
}
#[test]
fn mapped_parser_tokens_and_form_interaction_predicates() {
    use style::css_selector::{MatchType, PseudoType};
    use style::css_selector_list::CSSSelectorList;
    use style::parser::css_selector_parser::{CSSSelectorParser, SelectorParserContext};
    use style::persistent_selector::PersistentSelectorError;
    let context = SelectorParserContext {
        html: true,
        quirks: false,
    };
    let parse =
        |text: &str| CSSSelectorParser::ParseSelector(&foundation::String::from(text), &context);
    let list = CSSSelectorList::AdoptSelectorVector(parse(
        "main > p.a\\+b:nth-child(2n + 1), [title='a,b' i]",
    ));
    assert_eq!(list.ComplexSelectors().count(), 2);
    let first = list.First().unwrap();
    assert_eq!(
        first
            .SimpleSelectors()
            .find(|s| s.Match() == MatchType::kClass)
            .unwrap()
            .Value()
            .Utf8(),
        "a+b"
    );
    assert_eq!(
        first
            .SimpleSelectors()
            .find(|s| s.GetPseudoType() == PseudoType::kPseudoNthChild)
            .unwrap()
            .NthAValue(),
        2
    );
    for invalid in ["p >", "p,", "[foo=bar invalid]", ":has(:has(.x))"] {
        assert!(parse(invalid).is_empty(), "{invalid}");
    }
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let html = element(d, d.Root(), "html", "", "");
    let form = element(d, html, "form", "form", "");
    let input = element(d, form, "input", "field", "");
    for (name, value) in [
        ("placeholder", "hint"),
        ("required", ""),
        ("lang", "en-GB"),
        ("dir", "rtl"),
    ] {
        d.SetAttribute(
            input,
            DOMAttribute {
                local_name: name.into(),
                value: value.into(),
                ..Default::default()
            },
        );
    }
    let mut service = PersistentSelectorService::default();
    assert!(service
        .Matches(
            d,
            input,
            ":enabled:required:read-write:placeholder-shown:lang(en):dir(rtl)"
        )
        .unwrap());
    service.interaction.focused_node_id = Some(d.Node(input).Id());
    service.interaction.focus_visible_node_id = Some(d.Node(input).Id());
    service.interaction.hovered_node_id = Some(d.Node(input).Id());
    assert!(service
        .Matches(d, input, ":focus:focus-visible:hover")
        .unwrap());
    assert!(service.Matches(d, form, ":focus-within").unwrap());
    d.SetControlValue(input, "live".into());
    assert!(!service.Matches(d, input, ":placeholder-shown").unwrap());
    d.SetAttribute(
        input,
        DOMAttribute {
            local_name: "disabled".into(),
            value: "".into(),
            ..Default::default()
        },
    );
    assert!(service.Matches(d, input, ":disabled:read-only").unwrap());
    assert!(matches!(
        service.QueryFirst(d, form, ":playing"),
        Err(PersistentSelectorError::Unsupported(_))
    ));
    assert!(matches!(
        service.QueryAll(d, form, ":unknown"),
        Err(PersistentSelectorError::Syntax(_))
    ));
}

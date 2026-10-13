//! Regression coverage for the document's shared selector identity owner.
use super::*;
use dom::persistent_document::{DOMAttribute, DOMShadowRootMode};

fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: 800.0,
        viewport_height: 600.0,
        ..Default::default()
    }
}

fn node(owner: &DOM, id: &str) -> usize {
    let document = owner.GetDocument();
    (0..document.NodeCount())
        .find(|&index| {
            document
                .Node(index)
                .FindAttribute("id")
                .is_some_and(|attribute| attribute.value == id)
        })
        .unwrap()
}

fn attribute(owner: &mut DOM, element: usize, name: &str, value: &str) {
    owner.GetDocumentMut().SetAttribute(
        element,
        DOMAttribute {
            local_name: name.into(),
            value: value.into(),
            ..Default::default()
        },
    );
}

fn style(owner: &DOM, element: usize) -> &ComputedStyle {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get()
    }
}

fn handle(engine: &DocumentStyleEngine, owner: &DOM, element: usize) -> Rc<usize> {
    engine
        .selectors
        .borrow_mut()
        .StyleMatchSession(owner.GetDocument())
        .Backend()
        .handle(element)
}

fn update(engine: &mut DocumentStyleEngine, owner: &mut DOM) {
    engine.Update(owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
}

#[test]
fn shared_selector_owner_reuses_handles_after_growth_mutation_and_interaction() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>\
         .item{width:10px;height:2px;margin-left:0px}\
         .changed{width:20px} #marked{width:30px}\
         .item:hover{height:7px} .item:focus{height:9px}\
         .item:first-child{margin-left:4px}\
         main:has(> .changed){padding-left:3px}\
         </style><main id=root><div id=first class=item></div>\
         <div id=second class=item></div></main>",
    );
    let root = node(&owner, "root");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let first_id = owner.GetDocument().Node(first).Id();
    let second_id = owner.GetDocument().Node(second).Id();
    let mut engine = DocumentStyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    assert!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_applied
            >= 2
    );
    let selectors = engine.selectors.clone();
    let first_handle = handle(&engine, &owner, first);
    let second_handle = handle(&engine, &owner, second);
    assert_eq!(style(&owner, first).Width().Pixels(), 10.0);
    assert_eq!(style(&owner, first).MarginLeft().Pixels(), 4.0);
    assert_eq!(style(&owner, second).MarginLeft().Pixels(), 0.0);

    // Streaming parser growth extends only the existing handle owner.
    let added = owner
        .GetDocumentMut()
        .CreateElementDefault(DOMNamespace::kHTML, "div".into());
    attribute(&mut owner, added, "class", "item");
    owner.GetDocumentMut().AppendChild(root, added);
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, added).Width().Pixels(), 10.0);
    assert_eq!(*handle(&engine, &owner, added), added);
    assert!(Rc::ptr_eq(&first_handle, &handle(&engine, &owner, first)));

    attribute(&mut owner, first, "class", "item changed");
    engine
        .SetUserInteractionState(
            &mut owner,
            UserInteractionState {
                hovered_node_id: Some(first_id),
                ..Default::default()
            },
        )
        .unwrap();
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, first).Width().Pixels(), 20.0);
    assert_eq!(style(&owner, first).Height().Pixels(), 7.0);
    assert_eq!(style(&owner, root).PaddingLeft().Pixels(), 3.0);

    // A fresh backend must see both changed interaction and cached-id input.
    attribute(&mut owner, first, "id", "marked");
    engine
        .SetUserInteractionState(
            &mut owner,
            UserInteractionState {
                hovered_node_id: Some(second_id),
                focused_node_id: Some(first_id),
                ..Default::default()
            },
        )
        .unwrap();
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, first).Width().Pixels(), 30.0);
    assert_eq!(style(&owner, first).Height().Pixels(), 9.0);
    assert_eq!(style(&owner, second).Height().Pixels(), 7.0);

    owner.GetDocumentMut().Remove(first);
    owner.GetDocumentMut().AppendChild(root, first);
    engine
        .SetUserInteractionState(&mut owner, UserInteractionState::default())
        .unwrap();
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, first).MarginLeft().Pixels(), 0.0);
    assert_eq!(style(&owner, second).MarginLeft().Pixels(), 4.0);
    assert_eq!(style(&owner, first).Height().Pixels(), 2.0);
    assert_eq!(style(&owner, second).Height().Pixels(), 2.0);
    assert!(Rc::ptr_eq(&selectors, &engine.selectors));
    assert!(Rc::ptr_eq(&first_handle, &handle(&engine, &owner, first)));
    assert!(Rc::ptr_eq(&second_handle, &handle(&engine, &owner, second)));
}

#[test]
fn shared_selector_sessions_refresh_scope_and_shadow_tree_state() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>.item{width:1px}\
         @scope (.active){.item{width:9px}}</style>\
         <section id=root class=active><span id=target class=item></span></section>\
         <section id=outside></section><x-host id=host></x-host>",
    );
    let root = node(&owner, "root");
    let target = node(&owner, "target");
    let outside = node(&owner, "outside");
    let host = node(&owner, "host");
    let mut engine = DocumentStyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    let original_handle = handle(&engine, &owner, target);
    assert_eq!(style(&owner, target).Width().Pixels(), 9.0);

    attribute(&mut owner, root, "class", "inactive");
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 1.0);
    attribute(&mut owner, root, "class", "active");
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 9.0);
    owner.GetDocumentMut().AppendChild(outside, target);
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 1.0);

    // Attach a new TreeScope after the selector service has already interned
    // the light DOM. Style and scope caches must come from the next session.
    let (shadow, shadow_style) = {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let sheet = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        document.AppendText(sheet, ".item{width:27px}");
        document.AppendChild(shadow, sheet);
        document.AppendChild(shadow, target);
        (shadow, sheet)
    };
    update(&mut engine, &mut owner);
    assert_eq!(owner.GetDocument().TreeScopeRoot(target), shadow);
    assert_eq!(style(&owner, target).Width().Pixels(), 27.0);
    owner
        .GetDocumentMut()
        .SetTextContent(shadow_style, ".item{width:31px}".into());
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 31.0);
    owner.GetDocumentMut().AppendChild(root, target);
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 9.0);
    assert!(Rc::ptr_eq(
        &original_handle,
        &handle(&engine, &owner, target)
    ));
}

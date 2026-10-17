//! Production resolver ownership, not the legacy compatibility cascade.
use super::*;
use dom::persistent_document::DOMAttribute;
fn node(owner: &DOM, id: &str) -> usize {
    (0..owner.GetDocument().NodeCount())
        .find(|&i| {
            owner
                .GetDocument()
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap()
}
fn update(engine: &mut DocumentStyleEngine, owner: &mut DOM) {
    let media = MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: 800.,
        viewport_height: 600.,
        ..Default::default()
    };
    engine.Update(owner, &media, &[]).unwrap();
    assert_eq!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_fallbacks,
        0
    );
}
fn width(owner: &DOM, id: &str) -> f64 {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node(owner, id))
            .unwrap()
            .native_style
            .Get()
    }
    .Width()
    .Pixels() as f64
}
fn key_pointer(engine: &DocumentStyleEngine, owner: &DOM, id: &str) -> *const u8 {
    engine.custom_properties[&node(owner, id)]
        .values
        .get_key_value("--v0")
        .unwrap()
        .0
        .as_ptr()
}
#[test]
fn production_inherited_variable_maps_share_backing_until_override() {
    let _heap = foundation::LayoutHeapScope::new();
    let declarations = (0..256)
        .map(|i| format!("--v{i}:{}px;", i + 1))
        .collect::<std::string::String>();
    let mut owner = html::html_parser::ParseHTML(&format!("<!doctype html><main id=root style='{declarations}'><div id=a style='width:var(--v0)'><span id=b style='width:var(--v0)'></span></div><div id=override style='--v0:9px;width:var(--v0)'><span id=c style='width:var(--v0)'></span></div><div id=sibling style='width:var(--v0)'></div></main>"));
    let mut engine = DocumentStyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    assert!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_applied
            >= 6
    );
    assert_eq!(width(&owner, "a"), 1.);
    assert_eq!(width(&owner, "override"), 9.);
    assert_eq!(width(&owner, "c"), 9.);
    let root_key = key_pointer(&engine, &owner, "root");
    for id in ["a", "b", "sibling"] {
        assert_eq!(
            root_key,
            key_pointer(&engine, &owner, id),
            "ordinary production descendant must share the inherited map: {id}"
        );
    }
    assert_ne!(root_key, key_pointer(&engine, &owner, "override"));
    assert_eq!(
        key_pointer(&engine, &owner, "override"),
        key_pointer(&engine, &owner, "c")
    );
    let previous = engine.custom_properties[&node(&owner, "root")].clone();
    let sibling = engine.custom_properties[&node(&owner, "sibling")].clone();
    assert!(CustomPropertiesEquivalent(&previous, &sibling));
    let mut equal_copy = (*previous).clone();
    equal_copy.values = Rc::new((*previous.values).clone());
    assert!(!Rc::ptr_eq(&previous.values, &equal_copy.values));
    assert!(CustomPropertiesEquivalent(&previous, &equal_copy));
    assert!(!CustomPropertiesEquivalent(
        &previous,
        &engine.custom_properties[&node(&owner, "override")]
    ));

    let root = node(&owner, "root");
    owner.GetDocumentMut().SetAttribute(
        root,
        DOMAttribute {
            local_name: "style".into(),
            value: format!("{declarations}--v0:13px;"),
            ..Default::default()
        },
    );
    update(&mut engine, &mut owner);
    assert_eq!(width(&owner, "a"), 13.);
    assert_eq!(width(&owner, "b"), 13.);
    assert_eq!(width(&owner, "sibling"), 13.);
    assert_eq!(width(&owner, "override"), 9.);
    assert_eq!(width(&owner, "c"), 9.);
    assert_eq!(
        previous.Get("--v0").unwrap().original_text,
        foundation::String::from("1px")
    );
    assert!(!CustomPropertiesEquivalent(
        &previous,
        &engine.custom_properties[&root]
    ));
    for id in ["a", "b", "sibling"] {
        assert_eq!(
            key_pointer(&engine, &owner, "root"),
            key_pointer(&engine, &owner, id)
        );
    }
}
#[test]
fn production_registered_noninherited_reset_preserves_cow_and_values() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><main id=root style='--v0:1px;--local:23px'><div id=a style='width:var(--local)'><span id=b style='width:var(--local)'></span></div><div id=inherit style='--local:inherit;width:var(--local)'></div></main>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "@property --local{syntax:'<length>';inherits:false;initial-value:7px}",
    ));
    let mut engine = DocumentStyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    assert_eq!(width(&owner, "a"), 7.);
    assert_eq!(width(&owner, "b"), 7.);
    assert_eq!(width(&owner, "inherit"), 23.);
    assert_ne!(
        key_pointer(&engine, &owner, "root"),
        key_pointer(&engine, &owner, "a")
    );
    assert_eq!(
        key_pointer(&engine, &owner, "a"),
        key_pointer(&engine, &owner, "b"),
        "already-reset descendants retain immutable maps"
    );
    assert!(Rc::ptr_eq(
        &engine.custom_properties[&node(&owner, "a")].registered_values,
        &engine.custom_properties[&node(&owner, "b")].registered_values
    ));
    let child = &engine.custom_properties[&node(&owner, "a")];
    let mut different_registered = (**child).clone();
    different_registered.registered_values = engine.custom_properties[&node(&owner, "root")]
        .registered_values
        .clone();
    assert!(Rc::ptr_eq(&child.values, &different_registered.values));
    assert!(!CustomPropertiesEquivalent(child, &different_registered));
}

use dom::persistent_document::DOMAttribute;
use style::{media_queries::MediaValuesCachedData, StyleEngine};
fn media() -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: 800.0,
        viewport_height: 600.0,
        large_viewport_width: 800.0,
        large_viewport_height: 600.0,
        ..Default::default()
    }
}
fn node(owner: &dom::DOM, id: &str) -> usize {
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
#[test]
fn dom_attr_custom_inheritance_and_mutation_reach_native_computed_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=parent data-size=31><span id=child data-size=99></span></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#parent{--size:attr(DATA-SIZE px,7px);width:var(--size)}#child{width:var(--size);height:attr(data-size px);padding-top:attr(missing px,5px)}"));
    let parent = node(&owner, "parent");
    let child = node(&owner, "child");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    {
        let parent_style = owner.GetDocument().ResolvedStyleFor(parent).unwrap();
        let native = unsafe { &*parent_style.native_style.Get() };
        assert_eq!(native.Width().Pixels(), 31.0);
        assert!(native.HasAttrFunction());
        let child_style = owner.GetDocument().ResolvedStyleFor(child).unwrap();
        let native = unsafe { &*child_style.native_style.Get() };
        assert_eq!(native.Width().Pixels(), 31.0);
        assert_eq!(native.Height().Pixels(), 99.0);
        assert_eq!(native.PaddingTop().Pixels(), 5.0);
        assert!(native.HasAttrFunction());
    }
    owner.GetDocumentMut().SetAttribute(
        parent,
        DOMAttribute {
            local_name: "data-size".into(),
            value: "47".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let child_style = owner.GetDocument().ResolvedStyleFor(child).unwrap();
    let native = unsafe { &*child_style.native_style.Get() };
    assert_eq!(native.Width().Pixels(), 47.0);
    assert_eq!(native.Height().Pixels(), 99.0);
}
#[test]
fn invalid_attr_uses_typed_fallback_and_native_unset() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<div id=target data-size=12px data-ratio=.25></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#target{width:attr(data-size px,19px);height:attr(absent px);opacity:attr(data-ratio number);padding-top:attr(absent px,)}"));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert_eq!(native.Width().Pixels(), 19.0);
    assert!(native.Height().IsAuto());
    assert_eq!(native.Opacity(), 0.25);
    assert_eq!(native.PaddingTop().Pixels(), 0.0);
    assert!(native.HasAttrFunction());
}

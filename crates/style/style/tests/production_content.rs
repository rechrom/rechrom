use dom::persistent_document::DOMAttribute;
use layoutng_style::style::{computed_style_constants::PseudoId, content_data::TextContentData};
use style::{media_queries::MediaValuesCachedData, StyleEngine};

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
fn content(style: &layoutng_style::style::computed_style::ComputedStyle) -> String {
    let data = style.GetContentData().unwrap();
    assert!(unsafe { &*data }.IsText());
    assert!(unsafe { &*data }.Next().is_none());
    unsafe { &*(data as *const TextContentData) }
        .GetText()
        .Utf8()
}

#[test]
fn content_string_attr_important_and_mutation_use_native_data() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target data-label=one></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "#target{content:'origin'} #target::before{content:'wrong'} #target::before{content:attr(DATA-LABEL) ' \\41' !important} #target::before{content:'later'} #target::after{content:none}"));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    {
        let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
        let native = unsafe { &*resolved.native_style.Get() };
        assert_eq!(content(native), "origin");
        assert_eq!(
            content(unsafe { &*resolved.before.as_ref().unwrap().native_style.Get() }),
            "one A"
        );
        let cached = native.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore);
        assert!(unsafe { &*cached }.HasAttrFunction());
        let after = native.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdAfter);
        let after_data = unsafe { &*after }.GetContentData().unwrap();
        assert!(unsafe { &*after_data }.IsNone());
    }
    owner.GetDocumentMut().SetAttribute(
        target,
        DOMAttribute {
            local_name: "data-label".into(),
            value: "two".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(
        content(unsafe { &*resolved.before.as_ref().unwrap().native_style.Get() }),
        "two A"
    );
}

#[test]
fn content_inherit_is_chromium_noop_and_revert_layer_uses_normal_cascade() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "@layer base, override; #target{content:'origin'} #target::after{content:'earlier';content:inherit !important} @layer base{#target::before{content:'base'}} @layer override{#target::before{content:'override';content:revert-layer}}"));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    assert_eq!(content(native), "origin");
    assert_eq!(
        content(unsafe { &*resolved.before.as_ref().unwrap().native_style.Get() }),
        "base"
    );
    assert!(resolved.after.is_none());
    let inherited = native.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdAfter);
    assert!(unsafe { &*inherited }.GetContentData().is_none());
    assert!(unsafe { &*inherited }.HasExplicitInheritance());
    assert!(native.ChildHasExplicitInheritance());
}

use crate::{
    media_queries::MediaValuesCachedData, ParseCSS, ParseUserAgentCSSWithRole, StyleEngine,
};
use cssom::css_style_sheet::CSSUserAgentStyleSheetRole;
use dom::persistent_document::{DOMAttribute, DOMNamespace};
use foundation::Length;
use layoutng_style::style::computed_style::ComputedStyle;

fn node(owner: &dom::DOM, id: &str) -> usize {
    let doc = owner.GetDocument();
    (0..doc.NodeCount())
        .find(|&i| {
            doc.Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap()
}
fn computed_style<'a>(owner: &'a dom::DOM, id: &str) -> &'a ComputedStyle {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node(owner, id))
            .unwrap()
            .computed_style_owner
            .Get()
    }
}

#[test]
fn svg_hints_survive_production_inheritance_cascade_and_mutation() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r##"<!doctype html><svg fill="none" color="#123456">
        <g id="group" stroke="currentColor" stroke-width="2">
            <path id="inherited" d="M0 0L10 10"/>
            <path id="author" fill="red" stroke-width="3" style="stroke-width:5"/>
            <path id="invalid" fill="red;stroke:blue"/>
            <path id="namespaced"/>
        </g></svg><div id="html" fill="red"></div>"##,
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(ParseCSS("#author { fill:blue; stroke-width:4 }"));
    let namespaced = node(&owner, "namespaced");
    owner.GetDocumentMut().SetAttribute(
        namespaced,
        DOMAttribute {
            namespace_uri: "urn:test".into(),
            prefix: "other".into(),
            local_name: "fill".into(),
            value: "red".into(),
        },
    );
    let ua = vec![
        ParseUserAgentCSSWithRole(
            "html,body,div { display:block }",
            CSSUserAgentStyleSheetRole::kHTML,
        ),
        ParseUserAgentCSSWithRole("svg { display:inline }", CSSUserAgentStyleSheetRole::kSVG),
    ];
    let media = MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        em_size: 16.0,
        viewport_width: 800.0,
        viewport_height: 600.0,
        ..Default::default()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media, &ua).unwrap();
    assert!(computed_style(&owner, "inherited").FillPaint().IsNone());
    assert!(computed_style(&owner, "inherited")
        .StrokePaint()
        .HasCurrentColor());
    assert_eq!(
        computed_style(&owner, "inherited").StrokeWidth().length(),
        &Length::Fixed(2)
    );
    assert_eq!(
        computed_style(&owner, "inherited")
            .Color()
            .GetColor()
            .Param0(),
        0x12 as f32
    );
    assert_eq!(
        computed_style(&owner, "author")
            .FillPaint()
            .GetColor()
            .GetColor()
            .Param2(),
        255.0
    );
    assert_eq!(
        computed_style(&owner, "author").StrokeWidth().length(),
        &Length::Fixed(5)
    );
    assert!(computed_style(&owner, "invalid").FillPaint().IsNone());
    assert!(computed_style(&owner, "invalid")
        .StrokePaint()
        .HasCurrentColor());
    assert!(computed_style(&owner, "namespaced").FillPaint().IsNone());
    assert_eq!(
        owner.GetDocument().Node(node(&owner, "html")).Namespace(),
        DOMNamespace::kHTML
    );
    assert_eq!(
        computed_style(&owner, "html")
            .FillPaint()
            .GetColor()
            .GetColor()
            .Param0(),
        0.0
    );
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 8, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_failures, 0, "{stats:?}");

    let group = node(&owner, "group");
    owner.GetDocumentMut().SetAttribute(
        group,
        DOMAttribute {
            local_name: "stroke-width".into(),
            value: "7".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media, &ua).unwrap();
    assert_eq!(
        computed_style(&owner, "inherited").StrokeWidth().length(),
        &Length::Fixed(7)
    );
    assert_eq!(
        computed_style(&owner, "author").StrokeWidth().length(),
        &Length::Fixed(5)
    );
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(group, "stroke-width");
    engine.Update(&mut owner, &media, &ua).unwrap();
    assert_eq!(
        computed_style(&owner, "inherited").StrokeWidth().length(),
        &Length::Fixed(1)
    );
}

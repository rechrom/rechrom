//! Native feature-values ownership through Style's retained document lifecycle.
use font_engine::fonts::font_description::FontDescription;
use foundation::Persistent;
use layoutng_style::style::computed_style::ComputedStyle;
use style::{media_queries::MediaValuesCachedData, StyleEngine};

fn media(width: f64) -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: width,
        viewport_height: 600.,
        large_viewport_width: width,
        large_viewport_height: 600.,
        ..Default::default()
    }
}
fn node(owner: &dom::DOM, id: &str) -> usize {
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find(|&i| d.Node(i).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap()
}
fn native(owner: &dom::DOM, id: &str) -> Persistent<ComputedStyle> {
    owner
        .GetDocument()
        .ResolvedStyleFor(node(owner, id))
        .unwrap()
        .native_style
        .clone()
}
fn features(description: &FontDescription, family: &str) -> Vec<(String, u32)> {
    description
        .ResolvedFontFeaturesForFamilies()
        .iter()
        .find(|(name, _)| name.Utf8() == family)
        .unwrap_or_else(|| {
            panic!(
                "missing family {family}, entries: {:?}",
                description.ResolvedFontFeaturesForFamilies()
            )
        })
        .1
        .iter()
        .map(|f| {
            (
                String::from_utf8(f.tag.to_be_bytes().to_vec()).unwrap(),
                f.value,
            )
        })
        .collect()
}
fn actual(owner: &dom::DOM, id: &str, family: &str) -> Vec<(String, u32)> {
    features(
        unsafe { &*native(owner, id).Get() }.GetFontDescription(),
        family,
    )
}
fn update(
    engine: &mut StyleEngine,
    owner: &mut dom::DOM,
    width: f64,
) -> dom::style_state::StyleUpdateImpact {
    let impact = engine.Update(owner, &media(width), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    impact
}
fn ss(value: u8) -> Vec<(String, u32)> {
    vec![(format!("ss{value:02}"), 1)]
}

#[test]
fn symbolic_alternates_remain_canonical_with_distinct_family_resolution_and_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r#"<style>
      @font-feature-values Alpha { @styleset { Readable: 2 4 } @swash { flow: 3 } @stylistic { fancy: 5 } @ornaments { leaf: 6 } @annotation { note: 7 } @character-variant { variant: 8 9 } }
      @font-feature-values Beta { @styleset { Readable: 10 } }
      #parent { font-family: Alpha, Beta, Missing; font-variant-alternates: styleset(Readable) swash(flow) stylistic(fancy) ornaments(leaf) annotation(note) character-variant(variant) historical-forms }
      #parent::before { content: 'a'; font-family: Beta }
      #normal {font-variant-alternates: normal}
    </style><div id=parent><span id=child></span><span id=normal></span></div>"#,
    );
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    let expected = [
        ("swsh", 3),
        ("cswh", 3),
        ("ornm", 6),
        ("nalt", 7),
        ("salt", 5),
        ("ss02", 1),
        ("ss04", 1),
        ("cv08", 9),
        ("hist", 1),
    ]
    .map(|(t, v)| (t.to_string(), v))
    .to_vec();
    assert_eq!(actual(&owner, "parent", "Alpha"), expected);
    assert_eq!(actual(&owner, "child", "Alpha"), expected);
    assert_eq!(
        actual(&owner, "parent", "Beta"),
        vec![("ss10".into(), 1), ("hist".into(), 1)]
    );
    assert_eq!(
        actual(&owner, "parent", "Missing"),
        vec![("hist".into(), 1)]
    );
    let retained = native(&owner, "parent");
    let parent = unsafe { &*retained.Get() };
    assert_eq!(
        parent
            .GetFontDescription()
            .FontVariantAlternatesValue()
            .unwrap()
            .Styleset()[0]
            .Utf8(),
        "Readable"
    );
    let pseudo = unsafe {
        &*parent.GetCachedPseudoElementStyleWithoutArgument(
            layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdBefore,
        )
    };
    assert_eq!(
        features(pseudo.GetFontDescription(), "Beta"),
        vec![("ss10".into(), 1), ("hist".into(), 1)]
    );
    assert!(unsafe { &*native(&owner, "normal").Get() }
        .GetFontDescription()
        .ResolvedFontFeaturesForFamilies()
        .is_empty());
}

#[test]
fn alias_only_sheet_edits_invalidate_fonts_and_preserve_old_native_snapshot() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<!doctype html><style id=aliases>@font-feature-values Probe{@styleset{readable:2}}</style><style>#target{font-family:Probe;font-variant-alternates:styleset(readable)}</style><span id=target>letters</span>");
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    let retained = native(&owner, "target");
    assert_eq!(actual(&owner, "target", "Probe"), ss(2));
    assert!(update(&mut engine, &mut owner, 800.).IsEmpty());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
    let aliases = node(&owner, "aliases");
    owner.GetDocumentMut().SetTextContent(
        aliases,
        "@font-feature-values Probe{@styleset{readable:7}}".into(),
    );
    let impact = update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(7));
    assert!(impact.layout);
    assert_eq!(
        features(unsafe { &*retained.Get() }.GetFontDescription(), "Probe"),
        ss(2)
    );
    assert!(
        unsafe { &*retained.Get() }.GetFontDescription()
            != unsafe { &*native(&owner, "target").Get() }.GetFontDescription()
    );
    owner.GetDocumentMut().Remove(aliases);
    assert!(update(&mut engine, &mut owner, 800.).layout);
    assert!(actual(&owner, "target", "Probe").is_empty());
    assert_eq!(
        features(unsafe { &*retained.Get() }.GetFontDescription(), "Probe"),
        ss(2)
    );
}

#[test]
fn layer_order_source_order_owner_media_and_nested_media_rebuild_aliases() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<!doctype html><style id=order>@layer early,late;</style><style>@layer late{@font-feature-values Probe{@styleset{readable:3}}}</style><style>@layer early{@font-feature-values Probe{@styleset{readable:2}}}</style><style id=conditional media='(min-width:700px)'>@media(max-width:900px){@font-feature-values Probe{@styleset{readable:4}}}</style><span id=target style='font-family:Probe;font-variant-alternates:styleset(readable)'>x</span>");
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(4));
    update(&mut engine, &mut owner, 1000.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(3));
    update(&mut engine, &mut owner, 600.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(3));
    let order = node(&owner, "order");
    owner
        .GetDocumentMut()
        .SetTextContent(order, "@layer late,early;".into());
    let impact = update(&mut engine, &mut owner, 600.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(2));
    assert!(impact.layout);
    let conditional = node(&owner, "conditional");
    owner.GetDocumentMut().SetAttribute(
        conditional,
        dom::persistent_document::DOMAttribute {
            local_name: "media".into(),
            value: "all".into(),
            ..Default::default()
        },
    );
    update(&mut engine, &mut owner, 600.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(4));
}

#[test]
fn insertion_reordering_and_removal_of_alias_only_sheets_recalculate_unchanged_declarations() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<!doctype html><style id=first>@font-feature-values Probe{@styleset{readable:2}}</style><style id=second>@font-feature-values Probe{@styleset{readable:3}}</style><span id=target style='font-family:Probe;font-variant-alternates:styleset(readable)'>x</span>");
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(3));
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let parent = owner.GetDocument().Node(first).Parent().unwrap();
    owner.GetDocumentMut().InsertBefore(parent, second, first);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(2));
    owner.GetDocumentMut().Remove(first);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(3));
    owner.GetDocumentMut().AppendChild(parent, first);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "target", "Probe"), ss(2));
}

#[test]
fn document_author_scope_and_unicode_family_folding_keep_alias_names_case_sensitive() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<!doctype html><style>@font-feature-values STRASSE{@styleset{Readable:2;readable:3}}</style><div id=host></div><span id=outside style='font-family:Straße;font-variant-alternates:styleset(Readable)'>x</span>");
    let host = node(&owner, "host");
    let shadow = owner
        .GetDocumentMut()
        .AttachShadowRoot(host, dom::persistent_document::DOMShadowRootMode::kOpen);
    let rules = owner.GetDocumentMut().CreateElementDefault(
        dom::persistent_document::DOMNamespace::kHTML,
        "style".into(),
    );
    owner.GetDocumentMut().AppendText(rules,"@font-feature-values STRASSE{@styleset{Readable:8;readable:9}} #inside{font-family:Straße;font-variant-alternates:styleset(readable)}");
    owner.GetDocumentMut().AppendChild(shadow, rules);
    let inside = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "span".into());
    owner.GetDocumentMut().SetAttribute(
        inside,
        dom::persistent_document::DOMAttribute {
            local_name: "id".into(),
            value: "inside".into(),
            ..Default::default()
        },
    );
    owner.GetDocumentMut().AppendChild(shadow, inside);
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(actual(&owner, "outside", "Straße"), ss(2));
    assert_eq!(actual(&owner, "inside", "Straße"), ss(3));
}

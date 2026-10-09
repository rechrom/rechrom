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
fn width(owner: &dom::DOM, id: &str) -> f64 {
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
fn update(engine: &mut StyleEngine, owner: &mut dom::DOM, width: f64) {
    engine.Update(owner, &media(width), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}
#[test]
fn registered_custom_properties_apply_inherits_initial_invalid_and_dependency_fallback() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=parent style='--inherited:31px;--local:41px'><div id=child style='width:var(--inherited);height:var(--local)'></div><div id=invalid style='--inherited:red;--local:red;--copy:var(--local);width:var(--inherited);height:var(--copy)'></div><div id=inherit style='--local:inherit;width:var(--local)'></div><div id=initial style='--inherited:initial;width:var(--inherited)'></div><div id=unset style='--local:unset;width:var(--local)'></div></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("@property --inherited {syntax:'<length>';inherits:true;initial-value:7px} @property --local {syntax:'<length>';inherits:false;initial-value:11px}"));
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "child"), 31.);
    assert_eq!(width(&owner, "invalid"), 31.);
    for id in ["child", "invalid"] {
        assert_eq!(
            unsafe {
                &*owner
                    .GetDocument()
                    .ResolvedStyleFor(node(&owner, id))
                    .unwrap()
                    .native_style
                    .Get()
            }
            .Height()
            .Pixels(),
            11.
        );
    }
    assert_eq!(width(&owner, "inherit"), 41.);
    assert_eq!(width(&owner, "initial"), 7.);
    assert_eq!(width(&owner, "unset"), 11.);
    let version = engine.PropertyRegistry().Version();
    update(&mut engine, &mut owner, 800.);
    assert_eq!(engine.PropertyRegistry().Version(), version);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
}
#[test]
fn registered_font_relative_values_compute_before_inheritance_and_initial_units_are_absolute() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=parent style='font-size:20px;--size:2em'><div id=child style='font-size:10px;width:var(--size)'></div><div id=own style='font-size:30px;--size:2em;width:var(--size)'></div><div id=initial style='--size:initial;width:var(--size)'></div></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "@property --size {syntax:'<length>';inherits:true;initial-value:1in}",
    ));
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "child"), 40.);
    assert_eq!(width(&owner, "own"), 60.);
    assert_eq!(width(&owner, "initial"), 96.);
}
#[test]
fn property_rules_rebuild_for_media_sheet_source_order_and_layer_changes() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<div id=target style='width:var(--size,3px)'></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("@layer early, late;@layer late{@property --size{syntax:'<length>';inherits:false;initial-value:21px}}@layer early{@property --size{syntax:'<length>';inherits:false;initial-value:11px}}@media(min-width:700px){@property --size{syntax:'<length>';inherits:false;initial-value:31px}@property --size{syntax:'<length>';inherits:false;initial-value:41px}}"));
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 41.);
    let version = engine.PropertyRegistry().Version();
    update(&mut engine, &mut owner, 600.);
    assert_eq!(width(&owner, "target"), 21.);
    assert!(engine.PropertyRegistry().Version() > version);
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
        "@property --size{syntax:'<length>';inherits:false;initial-value:51px}",
    ));
    update(&mut engine, &mut owner, 600.);
    assert_eq!(width(&owner, "target"), 51.);
}
#[test]
fn registered_style_queries_use_computed_typed_values_and_registered_initial() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=container style='--size:1in;--angle:.5turn'><span id=target></span></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("@property --size{syntax:'<length>';inherits:false;initial-value:96px}@property --angle{syntax:'<angle>';inherits:false;initial-value:0deg}#target{width:1px;height:2px}@container style(--size:96px){#target{width:33px}}@container style(--angle:180deg){#target{height:44px}}@container style(--size){#target{width:99px}}"));
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 33.);
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(node(&owner, "target"))
                .unwrap()
                .native_style
                .Get()
        }
        .Height()
        .Pixels(),
        44.
    );
}
#[test]
fn stylesheet_registration_removal_recalculates_unregistered_inheritance() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<style id=rules>@property --size{syntax:'<length>';inherits:false;initial-value:7px}</style><div id=parent style='--size:31px'><div id=child style='width:var(--size)'></div></div>");
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "child"), 7.);
    let rules = node(&owner, "rules");
    owner.GetDocumentMut().Remove(rules);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "child"), 31.);
    assert!(engine.PropertyRegistry().Registration("--size").is_none());
}
#[test]
fn typed_script_registration_masks_declarations_and_survives_sheet_changes() {
    use style::{
        css_syntax_string_parser::CSSSyntaxStringParser,
        css_value::CSSValuePayload,
        parser::{
            css_parser_mode::CSSParserMode, production_property_parser::ParseDeclarationList,
        },
        property_registry::PropertyRegistration,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<style id=rules>@property --size{syntax:'<length>';inherits:false;initial-value:7px}</style><div id=target style='--copy:var(--size);width:var(--copy)'></div>");
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 7.);
    let parsed = ParseDeclarationList(
        &foundation::String::from("--size:17px"),
        CSSParserMode::kHTMLStandardMode,
    );
    let initial_value = parsed.properties[0].ValueRef();
    let CSSValuePayload::kUnparsedDeclarationClass(initial) = initial_value.Payload() else {
        panic!("retained declaration data")
    };
    let syntax = CSSSyntaxStringParser::new(&foundation::String::from("<length>"))
        .Parse()
        .unwrap();
    let registration =
        PropertyRegistration::new(syntax, false, Some(initial.data.clone())).unwrap();
    engine
        .RegisterProperty(
            &mut owner,
            foundation::AtomicString::from_str("--size"),
            registration.clone(),
        )
        .unwrap();
    assert!(engine
        .RegisterProperty(
            &mut owner,
            foundation::AtomicString::from_str("--size"),
            registration
        )
        .is_err());
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 17.);
    assert!(engine
        .PropertyRegistry()
        .IsInRegisteredPropertySet("--size"));
    assert!(engine.PropertyRegistry().WasReferenced("--size"));
    let rules = node(&owner, "rules");
    owner.GetDocumentMut().SetTextContent(
        rules,
        "@property --size{syntax:'<length>';inherits:false;initial-value:37px}".into(),
    );
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 17.);
    owner.GetDocumentMut().Remove(rules);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 17.);
}
#[test]
fn replacing_layer_order_rebuilds_registry_and_typed_color_queries_match() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<style id=rules>@layer early,late;@layer early{@property --size{syntax:'<length>';inherits:false;initial-value:11px}}@layer late{@property --size{syntax:'<length>';inherits:false;initial-value:21px}}</style><div id=container style='--tone:red'><span id=target style='width:var(--size)'></span></div>");
    owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("@property --tone{syntax:'<color>';inherits:true;initial-value:black}@container style(--tone:rgb(255,0,0)){#target{height:43px}}"));
    let mut engine = StyleEngine::new(&owner);
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 21.);
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(node(&owner, "target"))
                .unwrap()
                .native_style
                .Get()
        }
        .Height()
        .Pixels(),
        43.
    );
    let rules = node(&owner, "rules");
    owner.GetDocumentMut().SetTextContent(rules,"@layer late,early;@layer early{@property --size{syntax:'<length>';inherits:false;initial-value:11px}}@layer late{@property --size{syntax:'<length>';inherits:false;initial-value:21px}}".into());
    update(&mut engine, &mut owner, 800.);
    assert_eq!(width(&owner, "target"), 11.);
}

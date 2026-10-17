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
         .item:hover::after{content:'+'}\
         .item:hover ~ .item{margin-right:6px}\
         main:has(> .item:hover){padding-right:3px}\
         .item:first-child{margin-left:4px}\
         main:has(> .changed){padding-left:3px}\
         </style><main id=root><div id=first class=item></div>\
         <div id=second class=item></div></main>\
         <aside id=unrelated><p>stable</p><p>stable</p><p>stable</p></aside>",
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
    let unrelated = node(&owner, "unrelated");
    let retained = owner
        .GetDocument()
        .ResolvedStyleFor(unrelated)
        .unwrap()
        .native_style
        .clone();
    engine
        .SetUserInteractionState(
            &mut owner,
            UserInteractionState {
                hovered_node_id: Some(second_id),
                ..Default::default()
            },
        )
        .unwrap();
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, second).Height().Pixels(), 7.0);
    assert_eq!(style(&owner, first).MarginRight().Pixels(), 6.0);
    assert_eq!(style(&owner, root).PaddingRight().Pixels(), 3.0);
    assert_eq!(
        retained.Get(),
        owner
            .GetDocument()
            .ResolvedStyleFor(unrelated)
            .unwrap()
            .native_style
            .Get(),
        "unrelated styles must retain their canonical owner across hover changes"
    );
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

    let unrelated_style = owner
        .GetDocument()
        .ResolvedStyleFor(root)
        .unwrap()
        .native_style
        .clone();
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
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(root)
            .unwrap()
            .native_style
            .Get(),
        unrelated_style.Get()
    );
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

    // Connected traversal must include shadow scopes and hidden elements,
    // while detached hosts and their descendants stay outside invalidation.
    let detached = owner
        .GetDocumentMut()
        .CreateElementDefault(DOMNamespace::kHTML, "x-detached".into());
    let detached_shadow = owner
        .GetDocumentMut()
        .AttachShadowRoot(detached, DOMShadowRootMode::kOpen);
    let child = owner
        .GetDocumentMut()
        .CreateElementDefault(DOMNamespace::kHTML, "span".into());
    owner.GetDocumentMut().AppendChild(detached_shadow, child);
    attribute(&mut owner, shadow_style, "hidden", "");
    let document = owner.GetDocument();
    let connected = ConnectedElements(document).collect::<Vec<_>>();
    let expected = (0..document.NodeCount())
        .filter(|&index| ConnectedElement(document, index))
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        connected.len(),
        expected.len(),
        "each active element is visited once"
    );
    assert_eq!(
        connected
            .iter()
            .copied()
            .collect::<std::collections::HashSet<_>>(),
        expected
    );
    assert!(connected.contains(&shadow_style));
    assert!(!connected.contains(&child));

    // The invalidator can keep a fact while visiting additional elements;
    // sparse storage growth cannot revoke its stable typed owner.
    use crate::invalidation::invalidation_dom::InvalidationDom;
    let dom = crate::persistent_invalidation::PersistentInvalidationDom::new(document);
    let held = dom.element(&target).unwrap();
    for index in connected {
        let _ = dom.element(&index);
    }
    assert!(std::ptr::eq(&*held, &*dom.element(&target).unwrap()));
}

#[test]
fn canonical_candidate_signatures_preserve_cascade_and_diagnostic_fallbacks() {
    let _heap = foundation::LayoutHeapScope::new();
    let a = RetainedRuleSignature::new(vec![(2, 10, 9), (1, 100, 8)], true);
    assert!(a.Equivalent(&RetainedRuleSignature::new(
        vec![(1, 100, 8), (2, 10, 9)],
        true
    )));
    for (values, complete) in [
        (vec![(1, 100, 8), (2, 10, 9)], false),
        (vec![(1, 100, 8), (2, 11, 9)], true),
        (vec![(1, 100, 8), (2, 10, 8)], true),
        (vec![(1, 100, 8), (2, 10, 9), (2, 10, 9)], true),
    ] {
        assert!(!a.Equivalent(&RetainedRuleSignature::new(values, complete)));
    }
    let mut owner = html::html_parser::ParseHTML("<!doctype html><style>#target{width:20px}.hit{width:10px}.never .hit{height:5px}</style><main id=root><div id=target class=hit></div></main>");
    let target = node(&owner, "target");
    let root = node(&owner, "root");
    let mut engine = DocumentStyleEngine::new(&owner);
    update(&mut engine, &mut owner);
    assert!(engine.matches[&target].complete);
    assert_eq!(style(&owner, target).Width().Pixels(), 20.);
    let inputs_owner = engine.active_sheets.clone();
    let inputs = inputs_owner
        .iter()
        .map(|(sheet, origin)| (sheet.as_ref(), *origin))
        .collect::<Vec<_>>();
    let probe = engine.MatchingRules(
        owner.GetDocument(),
        target,
        &media(),
        &inputs,
        None,
        None,
        None,
    );
    assert!(
        engine.matches[&target].Equivalent(&RetainedRuleSignature::new(probe.clone(), true)),
        "retained={:?}, probe={probe:?}",
        engine.matches[&target].values
    );

    attribute(&mut owner, root, "class", "unrelated");
    update(&mut engine, &mut owner);
    assert_eq!(style(&owner, target).Width().Pixels(), 20.);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);

    // Global-to-ordinary transitions must not upgrade an unresolved empty
    // signature. This also covers inactive/global rules outside AllRuleData.
    for global in [
        "@function --pick(){result:1px;}",
        "@layer theme;",
        "@media print{.absent{width:1px}}",
    ] {
        let mut changing = html::html_parser::ParseHTML(&format!(
            "<!doctype html><style id=rules>{global}</style><div id=empty></div>"
        ));
        let rules = node(&changing, "rules");
        let empty = node(&changing, "empty");
        let mut changing_engine = DocumentStyleEngine::new(&changing);
        update(&mut changing_engine, &mut changing);
        assert!(!changing_engine.matches[&empty].complete, "{global}");
        assert!(changing_engine.matches[&empty].values.is_empty());
        changing
            .GetDocumentMut()
            .SetTextContent(rules, ".absent{width:2px}".into());
        update(&mut changing_engine, &mut changing);
        assert!(
            !changing_engine.matches[&empty].complete,
            "unresolved upgrade: {global}"
        );
        let sheets_owner = changing_engine.active_sheets.clone();
        let sheets = sheets_owner
            .iter()
            .map(|(sheet, origin)| (sheet.as_ref(), *origin))
            .collect::<Vec<_>>();
        assert!(changing_engine.CandidateRequiresRecalc(
            changing.GetDocument(),
            empty,
            &media(),
            &sheets,
            None,
            false,
            true,
            true
        ));
        attribute(&mut changing, empty, "class", "unrelated");
        update(&mut changing_engine, &mut changing);
        assert!(changing_engine.matches[&empty].complete);
        assert!(!changing_engine.CandidateRequiresRecalc(
            changing.GetDocument(),
            empty,
            &media(),
            &sheets,
            None,
            false,
            true,
            true
        ));
        assert!(changing_engine.CandidateRequiresRecalc(
            changing.GetDocument(),
            empty,
            &media(),
            &sheets,
            None,
            false,
            false,
            true
        ));
    }

    // Built-in UA special pseudos remain conservative. The warm discarded
    // ordinary probe may be omitted, but ordinary resolution must still run.
    let ua = vec![crate::ParseUserAgentCSS(include_str!(
        "../../browser/resources/user_agent/html.css"
    ))];
    engine.Update(&mut owner, &media(), &ua).unwrap();
    assert!(!engine.matches[&target].complete);
    let sheets_owner = engine.active_sheets.clone();
    let sheets = sheets_owner
        .iter()
        .map(|(sheet, origin)| (sheet.as_ref(), *origin))
        .collect::<Vec<_>>();
    assert!(
        !engine
            .rule_candidate_index
            .as_ref()
            .unwrap()
            .matching_may_report_diagnostics
    );
    engine.diagnostics.clear();
    let baseline = engine.MatchingRules(
        owner.GetDocument(),
        target,
        &media(),
        &sheets,
        None,
        None,
        None,
    );
    assert!(engine.diagnostics.is_empty());
    assert!(!engine.matches[&target].Equivalent(&RetainedRuleSignature::new(baseline, false)));
    assert!(engine.CandidateRequiresRecalc(
        owner.GetDocument(),
        target,
        &media(),
        &sheets,
        None,
        true,
        true,
        false
    ));
    assert!(engine.diagnostics.is_empty());
    if std::env::var_os("RECHROM_STYLE_CANDIDATE_TIMING").is_some() {
        let medium = media();
        let iterations = 2000;
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            let signatures = engine.MatchingRules(
                owner.GetDocument(),
                target,
                &medium,
                &sheets,
                None,
                None,
                None,
            );
            std::hint::black_box(
                engine.matches[&target].Equivalent(&RetainedRuleSignature::new(signatures, false)),
            );
        }
        let baseline = start.elapsed();
        let start = std::time::Instant::now();
        for _ in 0..iterations {
            assert!(std::hint::black_box(engine.CandidateRequiresRecalc(
                owner.GetDocument(),
                target,
                &medium,
                &sheets,
                None,
                true,
                true,
                false
            )));
        }
        eprintln!(
            "candidate helper iterations={iterations} baseline_ns={} guarded_ns={}",
            baseline.as_nanos(),
            start.elapsed().as_nanos()
        );
    }
    // The compiled invalidation owner can exclude unrelated class keys even
    // with an incomplete signature. Exercise an actual ancestor key here so
    // that the conservative candidate-resolution contract is still covered.
    attribute(&mut owner, root, "class", "never");
    engine.Update(&mut owner, &media(), &ua).unwrap();
    assert_eq!(style(&owner, target).Width().Pixels(), 20.);
    assert_eq!(style(&owner, target).Height().Pixels(), 5.);
    assert!(owner.GetDocument().StyleState().stats.resolved_nodes >= 2);
    // Conditional probes retain the old diagnostic path and multiplicity.
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS("@scope (.scope::before){.hit{width:21px}} @container style(--x: var(--y)){.hit{height:1px}}"));
    engine.Update(&mut owner, &media(), &ua).unwrap();
    let sheets_owner = engine.active_sheets.clone();
    let sheets = sheets_owner
        .iter()
        .map(|(sheet, origin)| (sheet.as_ref(), *origin))
        .collect::<Vec<_>>();
    engine.diagnostics.clear();
    engine.MatchingRules(
        owner.GetDocument(),
        target,
        &media(),
        &sheets,
        None,
        None,
        None,
    );
    assert!(
        engine
            .rule_candidate_index
            .as_ref()
            .unwrap()
            .matching_may_report_diagnostics
    );
    let diagnostics = format!("{:?}", engine.Diagnostics());
    engine.diagnostics.clear();
    assert!(engine.CandidateRequiresRecalc(
        owner.GetDocument(),
        target,
        &media(),
        &sheets,
        None,
        true,
        true,
        false
    ));
    assert_eq!(format!("{:?}", engine.Diagnostics()), diagnostics);
}

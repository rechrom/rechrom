use dom::dom_mutation::{ApplyDOMMutations, DOMMutation, DOMMutationType};
use dom::{Document, DOM};
use std::sync::Arc;
use style::*;

fn element(document: &Document, id: &str) -> usize {
    (0..document.NodeCount())
        .find(|&i| {
            document
                .Node(i)
                .FindAttribute("id")
                .is_some_and(|a| a.value == id)
        })
        .unwrap()
}
fn update(owner: &mut DOM) {
    update_with_environment(owner, &Default::default());
}
fn update_with_environment(
    owner: &mut DOM,
    environment: &media_queries::media_values_cached::MediaValuesCachedData,
) {
    let mut engine = StyleEngine::new(owner);
    engine
        .Update(owner, environment, &[])
        .expect("style update");
}
fn assert_full_recalc_equal(owner: &mut DOM) {
    let before: Vec<_> = (0..owner.GetDocument().NodeCount())
        .map(|i| owner.GetDocument().ResolvedStyleHandle(i))
        .collect();
    owner.GetDocumentMut().InvalidateAllStyles();
    update(owner);
    for (i, expected) in before.iter().enumerate() {
        assert!(
            expected.as_deref() == owner.GetDocument().ResolvedStyleFor(i),
            "incremental style differs at node {}",
            i
        );
    }
}
fn attribute(owner: &mut DOM, id: &str, name: &str, value: &str) {
    let node = element(owner.GetDocument(), id);
    let mutation = DOMMutation {
        mutation_type: DOMMutationType::kSetAttribute,
        target_node_id: owner.GetDocument().Node(node).Id(),
        name: name.into(),
        value: value.into(),
        ..Default::default()
    };
    ApplyDOMMutations(owner.GetDocumentMut(), &[mutation]);
}
fn fixture(css: &str) -> DOM {
    let mut owner = html::html_parser::ParseHTML("<html><body><section id=left><div id=a class=box><span id=leaf>text</span></div><div id=b class=box></div></section><section id=right><div id=c class=box><span id=other>text</span></div></section></body></html>");
    owner
        .GetDocumentMut()
        .AppendStyleSheet(style::ParseCSS(css));
    update(&mut owner);
    owner.GetDocumentMut().StyleStateMut().impact = Default::default();
    owner
}

#[test]
fn inline_style_without_selector_dependency_stays_local_and_matches_old_fallback() {
    crate::native_test_thread::run(|| {
        let css="body:has(.unrelated){color:blue}.box + .box{height:17px}.box{--w:10px;width:var(--w)}span{color:inherit}::view-transition-old(sb){opacity:0}html:active-view-transition-type(aimc)::view-transition-new(sb){opacity:0}";
        let mut optimized = fixture(css);
        let mut baseline = fixture(css);
        let other = element(optimized.GetDocument(), "c");
        let retained = optimized.GetDocument().ResolvedStyleHandle(other).unwrap();
        for value in [
            "width:45px;height:23px",
            "--w:29px;color:red",
            "display:none",
            "",
        ] {
            attribute(&mut optimized, "a", "style", value);
            attribute(&mut baseline, "a", "style", value);
            let a = element(baseline.GetDocument(), "a");
            // Reproduce the previous Node invalidation on the same frozen DOM
            // and CSS fixture; this still promotes through unrelated :has.
            baseline.GetDocumentMut().InvalidateNodeStyle(a);
            update(&mut optimized);
            update(&mut baseline);
            assert!(optimized.GetDocument().StyleState().stats.resolved_nodes <= 2);
            assert!(baseline.GetDocument().StyleState().stats.resolved_nodes > 2);
            for i in 0..optimized.GetDocument().NodeCount() {
                assert!(
                    optimized.GetDocument().ResolvedStyleFor(i)
                        == baseline.GetDocument().ResolvedStyleFor(i),
                    "node {i}, style {value}"
                );
            }
            assert!(Arc::ptr_eq(
                &retained,
                &optimized.GetDocument().ResolvedStyleHandle(other).unwrap()
            ));
        }
        attribute(&mut optimized, "a", "style", "");
        update(&mut optimized);
        assert_eq!(
            optimized.GetDocument().StyleState().stats.resolved_nodes,
            0,
            "ineffective style write retains exact state"
        );
        assert_full_recalc_equal(&mut optimized);
    });
}

#[test]
fn inline_style_selector_dependency_and_unknown_syntax_keep_full_fallback() {
    crate::native_test_thread::run(|| {
        let mut owner=fixture(".box{width:10px}body:has(:is(#a[style],.missing)) #b{width:99px}#a[style] + #b{height:44px}");
        attribute(&mut owner, "a", "style", "color:red");
        update(&mut owner);
        let b = element(owner.GetDocument(), "b");
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(b).unwrap().style.width,
            Some(99.)
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(b)
                .unwrap()
                .style
                .height,
            Some(44.)
        );
        assert!(owner.GetDocument().StyleState().stats.resolved_nodes > 2);
        assert_full_recalc_equal(&mut owner);
        let a = element(owner.GetDocument(), "a");
        owner.GetDocumentMut().RemoveAttributeDefault(a, "style");
        update(&mut owner);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(b).unwrap().style.width,
            Some(10.)
        );
        assert!(owner.GetDocument().StyleState().stats.resolved_nodes > 2);
        assert_full_recalc_equal(&mut owner);
        for unknown in [r"[st\79le]", "[ns|style]", ":future(.x)"] {
            let mut owner = fixture(&format!(
                "body:has(.missing){{color:red}}{unknown} + .box{{height:12px}}"
            ));
            attribute(&mut owner, "a", "style", "width:45px");
            update(&mut owner);
            assert!(
                owner.GetDocument().StyleState().stats.resolved_nodes > 2,
                "unknown selector {unknown} retains broad fallback"
            );
            assert_full_recalc_equal(&mut owner);
        }
    });
}

#[test]
fn inline_style_control_native_geometry_computed_style_and_pixels_match_old_fallback() {
    crate::native_test_thread::run(|| {
        use javascript::{
            javascript_runtime::JavaScriptRuntime,
            quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
        };
        use std::{cell::RefCell, rc::Rc};
        use webapi::dom_bindings::DOMJavaScriptBindings;
        fn run(old_fallback: bool) -> (Vec<paint::paint_engine::PaintRect>, Vec<u8>, usize) {
            let document=Rc::new(RefCell::new(html::html_parser::ParseHTML("<html><body style='margin:0'><textarea id=q style='box-sizing:border-box;border:0;padding:0;width:80px;height:20px'>abc</textarea><div id=other class=row>retained text</div><div class=row>other text</div></body></html>")));
            document.borrow_mut().GetDocumentMut().AppendStyleSheet(style::ParseCSS("body:has(.missing){color:red}.row + .row{width:70px}::view-transition-old(sb){opacity:0}html:active-view-transition-type(aimc)::view-transition-new(sb){opacity:0}"));
            let constraints = Rc::new(RefCell::new(crate::CreateBrowserConstraints(320, 200)));
            crate::style_services::ResolveLayoutStyles(
                &mut document.borrow_mut(),
                &constraints.borrow(),
            );
            let q = element(document.borrow().GetDocument(), "q");
            let qid = document.borrow().GetDocument().Node(q).Id();
            let interaction = dom::UserInteractionState::default();
            let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
            let before = crate::LayoutPersistentDocument(
                &mut layout_engine,
                &mut document.borrow_mut(),
                &interaction,
                &constraints.borrow(),
            );
            let old_rects = paint::paint_engine::FragmentClientRects(&before, qid);
            assert_eq!(old_rects[0].height, 20.);
            document
                .borrow_mut()
                .GetDocumentMut()
                .SetControlValue(q, "abcd".into());
            attribute(
                &mut document.borrow_mut(),
                "q",
                "style",
                "box-sizing:border-box;border:0;padding:0;width:80px;height:30px",
            );
            if old_fallback {
                document
                    .borrow_mut()
                    .GetDocumentMut()
                    .InvalidateNodeStyle(q);
            }
            crate::style_services::ResolveLayoutStyles(
                &mut document.borrow_mut(),
                &constraints.borrow(),
            );
            let resolved = document
                .borrow()
                .GetDocument()
                .StyleState()
                .stats
                .resolved_nodes;
            let host = crate::style_services::CreateLayoutBindingsHost(
                document.clone(),
                constraints.clone(),
                Rc::new(RefCell::new(interaction)),
            );
            let mutated = document.clone();
            let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
                document.clone(),
                Box::new(move |m| {
                    crate::dom_mutation::ApplyDOMTreeMutation(&mut mutated.borrow_mut(), m)
                }),
                host,
            )));
            let mut runtime = QuickJsJavaScriptRuntime::new();
            let realm = runtime.CreateRealm(bindings);
            let bootstrap = runtime.Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl");
            assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
            let checked=runtime.Evaluate(&realm,"const q=document.getElementById('q');if(q.value!=='abcd'||getComputedStyle(q).height!=='30px'||q.offsetHeight!==30||q.getClientRects().length!==1||q.getBoundingClientRect().height!==30)throw Error('native control CSSOM mismatch')","inline-style-native-cssom");
            assert!(checked.Succeeded(), "{:?}", checked.exception);
            let current = crate::LayoutPersistentDocument(
                &mut layout_engine,
                &mut document.borrow_mut(),
                &interaction,
                &constraints.borrow(),
            );
            let rects = paint::paint_engine::FragmentClientRects(&current, qid);
            assert_eq!(
                paint::paint_engine::FragmentClientRects(&before, qid),
                old_rects,
                "retained old geometry snapshot is immutable"
            );
            let pixels = raster::pure_replay::RasterizeDisplayItemList(
                &paint::paint_engine::Paint(&current),
                320,
                200,
            );
            (rects, pixels, resolved)
        }
        let (old_rects, old_pixels, old_resolved) = run(true);
        let (rects, pixels, resolved) = run(false);
        assert_eq!(rects, old_rects);
        assert_eq!(pixels, old_pixels);
        assert_eq!(resolved, 1);
        assert!(old_resolved > resolved);
    });
}

#[test]
fn no_op_and_local_mutation_preserve_rule_and_style_handles() {
    crate::native_test_thread::run(|| {
        let mut owner =
            fixture(".box{height:20px}.active .box{height:40px}.active span{color:red}");
        let c = element(owner.GetDocument(), "c");
        let a = element(owner.GetDocument(), "a");
        let old_c = owner.GetDocument().ResolvedStyleHandle(c).unwrap();
        let old_a = owner.GetDocument().ResolvedStyleHandle(a).unwrap();
        update(&mut owner);
        assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
        assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 0);
        attribute(&mut owner, "left", "class", "active");
        update(&mut owner);
        assert!(Arc::ptr_eq(
            &old_c,
            &owner.GetDocument().ResolvedStyleHandle(c).unwrap()
        ));
        assert!(!Arc::ptr_eq(
            &old_a,
            &owner.GetDocument().ResolvedStyleHandle(a).unwrap()
        ));
        assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 4);
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn sibling_selectors_and_inherited_variables_match_full_recalc() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box{--size:10px;width:var(--size)}.selected + .box{width:70px}.selected ~ .box span{color:blue}.selected{--size:30px}span{height:var(--size)}");
        attribute(&mut owner, "a", "class", "box selected");
        update(&mut owner);
        let b = element(owner.GetDocument(), "b");
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(b).unwrap().style.width,
            Some(70.0)
        );
        assert_full_recalc_equal(&mut owner);
        attribute(&mut owner, "a", "class", "box");
        update(&mut owner);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(b).unwrap().style.width,
            Some(10.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn child_order_empty_and_reparenting_match_full_recalc() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box:first-child{width:11px}.box:last-child{height:33px}.box:empty{width:44px}#right span{color:blue}");
        let a = element(owner.GetDocument(), "a");
        let left = element(owner.GetDocument(), "left");
        let right = element(owner.GetDocument(), "right");
        owner.GetDocumentMut().AppendChild(left, a);
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().SetTextContent(a, String::new());
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().AppendChild(right, a);
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().Remove(a);
        update(&mut owner);
        assert!(owner.GetDocument().ResolvedStyleFor(a).is_none());
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn stylesheet_media_and_hidden_subtree_updates_match_full_recalc() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box{width:10px}@media(min-width:500px){.box{width:80px}}");
        let a = element(owner.GetDocument(), "a");
        let env = media_queries::media_values_cached::MediaValuesCachedData {
            viewport_width: 600.0,
            viewport_height: 300.0,
            small_viewport_width: 600.0,
            small_viewport_height: 300.0,
            large_viewport_width: 600.0,
            large_viewport_height: 300.0,
            dynamic_viewport_width: 600.0,
            dynamic_viewport_height: 300.0,
            ..Default::default()
        };
        update_with_environment(&mut owner, &env);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(80.0)
        );
        assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 0);
        attribute(&mut owner, "left", "style", "display:none");
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        assert!(
            !owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .generates_box
        );
        attribute(&mut owner, "left", "style", "display:block");
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        assert!(
            owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .generates_box
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(".box{width:99px}"));
        update(&mut owner);
        assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 1);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(99.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn paint_only_and_ineffective_changes_report_correct_impact() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box{width:20px;color:black}.red{color:red}");
        attribute(&mut owner, "a", "class", "box red");
        update(&mut owner);
        let impact = owner.GetDocument().StyleState().impact;
        assert!(impact.paint);
        assert!(!impact.layout);
        assert!(!impact.reattach);
        owner.GetDocumentMut().StyleStateMut().impact = Default::default();
        attribute(&mut owner, "a", "class", "box red unused");
        update(&mut owner);
        assert!(owner.GetDocument().StyleState().impact.IsEmpty());
        attribute(&mut owner, "a", "style", "width:40px");
        update(&mut owner);
        assert!(owner.GetDocument().StyleState().impact.layout);
    });
}

#[test]
fn relational_selector_and_pseudo_content_match_full_recalc() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture("section:has(.selected){width:90px}section:has(.selected) + section{height:55px}.selected::before{content:'prefix';color:red}");
        attribute(&mut owner, "a", "class", "box selected");
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
        attribute(&mut owner, "a", "class", "box");
        update(&mut owner);
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn unchanged_inline_declarations_are_retained_and_edits_replace_them() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture("");
        attribute(&mut owner, "a", "style", "width:40px;color:red");
        update(&mut owner);
        let a = element(owner.GetDocument(), "a");
        let old = owner.GetDocument().ResolvedStyleHandle(a).unwrap();
        attribute(&mut owner, "a", "class", "unused");
        update(&mut owner);
        assert!(Arc::ptr_eq(
            &old,
            &owner.GetDocument().ResolvedStyleHandle(a).unwrap()
        ));
        attribute(&mut owner, "a", "style", "width:50px");
        update(&mut owner);
        assert!(!Arc::ptr_eq(
            &old,
            &owner.GetDocument().ResolvedStyleHandle(a).unwrap()
        ));
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn removing_and_reconnecting_stylesheet_owner_updates_other_subtrees() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box{width:10px}");
        let left = element(owner.GetDocument(), "left");
        let style = owner.GetDocumentMut().CreateElementDefault(
            dom::persistent_document::DOMNamespace::kHTML,
            "style".into(),
        );
        owner.GetDocumentMut().AppendChild(left, style);
        let mut sheet = style::ParseCSS(".box{width:90px}");
        sheet.owner_node_id = owner.GetDocument().Node(style).Id();
        owner.GetDocumentMut().AppendStyleSheet(sheet);
        update(&mut owner);
        let c = element(owner.GetDocument(), "c");
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(c).unwrap().style.width,
            Some(90.0)
        );
        owner.GetDocumentMut().Remove(style);
        update(&mut owner);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(c).unwrap().style.width,
            Some(10.0)
        );
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().AppendChild(left, style);
        update(&mut owner);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(c).unwrap().style.width,
            Some(90.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn animation_and_control_state_changes_use_the_same_invalidation_path() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture("span{color:inherit}");
        let a = element(owner.GetDocument(), "a");
        let c = element(owner.GetDocument(), "c");
        let old_c = owner.GetDocument().ResolvedStyleHandle(c).unwrap();
        let a_id = owner.GetDocument().Node(a).Id();
        owner.GetDocumentMut().SetAnimationStyle(
            a_id,
            1,
            style::ParseCSSDeclarationList("color:red;width:80px"),
        );
        update(&mut owner);
        assert!(Arc::ptr_eq(
            &old_c,
            &owner.GetDocument().ResolvedStyleHandle(c).unwrap()
        ));
        assert_full_recalc_equal(&mut owner);
        let left = element(owner.GetDocument(), "left");
        let input = owner.GetDocumentMut().CreateElementDefault(
            dom::persistent_document::DOMNamespace::kHTML,
            "input".into(),
        );
        owner.GetDocumentMut().SetAttribute(
            input,
            dom::persistent_document::DOMAttribute {
                local_name: "placeholder".into(),
                value: "search".into(),
                ..Default::default()
            },
        );
        owner.GetDocumentMut().AppendChild(left, input);
        owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
            "input:placeholder-shown{width:11px}input:not(:placeholder-shown){width:42px}",
        ));
        update(&mut owner);
        owner
            .GetDocumentMut()
            .SetControlValue(input, "typed".into());
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(input)
                .unwrap()
                .style
                .width,
            Some(42.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn image_completion_updates_consumers_without_recalculating_unrelated_styles() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box + .box{padding:2px}body:has(.active){color:red}#a{background-image:url(test:background)}#b{mask-image:url(test:mask)}#c::before{content:'icon';background-image:url(test:pseudo)}");
        let other = element(owner.GetDocument(), "other");
        let handle = owner.GetDocument().ResolvedStyleHandle(other).unwrap();
        let mut id = 1;
        for (source, consumer) in [
            ("test:background", "a"),
            ("test:mask", "b"),
            ("test:pseudo", "c"),
            ("test:unreferenced", ""),
        ] {
            owner.GetDocumentMut().SetImageResource(
                source.into(),
                dom::ImageResourceMetadata {
                    id,
                    natural_width: 10.0,
                    natural_height: 20.0,
                    resolution_scale: 1.0,
                },
            );
            assert!(!owner.GetDocument().StyleState().all_dirty);
            update(&mut owner);
            let stats = owner.GetDocument().StyleState().stats;
            assert_eq!(
                stats.resolved_nodes,
                usize::from(!consumer.is_empty()),
                "{source} updates only its consumer"
            );
            assert!(Arc::ptr_eq(
                &handle,
                &owner.GetDocument().ResolvedStyleHandle(other).unwrap()
            ));
            if !consumer.is_empty() {
                let style = owner
                    .GetDocument()
                    .ResolvedStyleFor(element(owner.GetDocument(), consumer))
                    .unwrap();
                let actual = match consumer {
                    "a" => style.style.paint.background_images[0].resource_id,
                    "b" => style.style.paint.mask_images[0].image.resource_id,
                    "c" => {
                        style.before.as_ref().unwrap().style.paint.background_images[0].resource_id
                    }
                    _ => unreachable!(),
                };
                assert_eq!(actual, id);
            } else {
                assert_eq!(stats.resolved_nodes, 0);
            }
            assert_full_recalc_equal(&mut owner);
            id += 1;
        }
    });
}

#[test]
#[ignore = "manual Debug resource and resident heap timing"]
fn profile_loading_resource_and_shared_heap() {
    crate::native_test_thread::run(|| {
        let mut html = String::from("<html><body>");
        for i in 0..1200 {
            html.push_str(&format!(
                "<div class='box c{}'><span>article {i}</span></div>",
                i % 30
            ));
        }
        html.push_str("</body></html>");
        let mut owner = html::html_parser::ParseHTML(&html);
        let mut css = String::from("body{background-image:url(test:loading)}.box + .box{padding:1px}body:has(.missing){color:red}");
        for i in 0..30 {
            css.push_str(&format!(".c{i}{{height:20px;color:rgb({i},10,20)}}"));
        }
        owner
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS(&css));
        update(&mut owner);
        let started = std::time::Instant::now();
        owner.GetDocumentMut().SetImageResource(
            "test:loading".into(),
            dom::ImageResourceMetadata {
                id: 77,
                natural_width: 10.0,
                natural_height: 20.0,
                resolution_scale: 1.0,
            },
        );
        update(&mut owner);
        eprintln!(
            "loading-resource-bench ms={:.3} resolved={}",
            started.elapsed().as_secs_f64() * 1000.0,
            owner.GetDocument().StyleState().stats.resolved_nodes
        );
        let cs = crate::CreateBrowserConstraints(1024, 768);
        let interaction = dom::UserInteractionState::default();
        let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let _large =
            crate::LayoutPersistentDocument(&mut layout_engine, &mut owner, &interaction, &cs);
        let mut toolbar =
            html::html_parser::ParseHTML("<html><body><input value='address'></body></html>");
        update(&mut toolbar);
        let mut toolbar_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let _first =
            crate::LayoutPersistentDocument(&mut toolbar_engine, &mut toolbar, &interaction, &cs);
        for i in 0..3 {
            let started = std::time::Instant::now();
            let frame = crate::LayoutPersistentDocument(
                &mut toolbar_engine,
                &mut toolbar,
                &interaction,
                &cs,
            );
            eprintln!(
                "shared-heap-toolbar-bench iteration={i} ms={:.3}",
                started.elapsed().as_secs_f64() * 1000.0
            );
            std::hint::black_box(frame);
        }
    });
}

#[test]
fn image_notification_does_not_hide_simultaneous_dom_invalidation() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture("#a{background-image:url(test:background)}.active{--size:29px;color:red}#leaf{width:var(--size,10px)}");
        owner.GetDocumentMut().SetImageResource(
            "test:background".into(),
            dom::ImageResourceMetadata {
                id: 23,
                natural_width: 10.0,
                natural_height: 20.0,
                resolution_scale: 1.0,
            },
        );
        attribute(&mut owner, "a", "class", "box active");
        update(&mut owner);
        let a = element(owner.GetDocument(), "a");
        let leaf = element(owner.GetDocument(), "leaf");
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .style
                .paint
                .background_images[0]
                .resource_id,
            23
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(leaf)
                .unwrap()
                .style
                .width,
            Some(29.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn image_completion_updates_intrinsic_size_and_keeps_unrelated_styles() {
    crate::native_test_thread::run(|| {
        fn find(
            root: &layoutng_assembly::fragment_tree::FragmentNode,
            id: u64,
        ) -> Option<&layoutng_assembly::fragment_tree::FragmentNode> {
            if root.node_id == id {
                return Some(root);
            }
            root.children.iter().find_map(|child| find(child, id))
        }
        let mut owner = html::html_parser::ParseHTML(
            "<html><body><img id=photo src='test:photo'><div id=other>text</div></body></html>",
        );
        update(&mut owner);
        let other = element(owner.GetDocument(), "other");
        let handle = owner.GetDocument().ResolvedStyleHandle(other).unwrap();
        let id = owner
            .GetDocument()
            .Node(element(owner.GetDocument(), "photo"))
            .Id();
        let mut cs = crate::CreateBrowserConstraints(320, 200);
        let interaction = dom::UserInteractionState::default();
        let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let _initial =
            crate::LayoutPersistentDocument(&mut layout_engine, &mut owner, &interaction, &cs);
        owner.GetDocumentMut().StyleStateMut().impact = Default::default();
        dom::image_resource::AddImageResource(
            &mut owner,
            &mut cs,
            "test:photo".into(),
            layoutng_assembly::internal::layout_input::PaintImage {
                id: 9,
                revision: 1,
                width: 128,
                height: 64,
                resolution_scale: 2.0,
                content: image_resource::PaintImageContent::Bitmap(vec![255; 128 * 64 * 4].into()),
            },
        );
        update(&mut owner);
        assert!(owner.GetDocument().StyleState().impact.layout);
        assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
        assert!(Arc::ptr_eq(
            &handle,
            &owner.GetDocument().ResolvedStyleHandle(other).unwrap()
        ));
        let current =
            crate::LayoutPersistentDocument(&mut layout_engine, &mut owner, &interaction, &cs);
        let image = find(&current, id).unwrap();
        assert_eq!((image.size.width, image.size.height), (64.0, 32.0));
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn coalesced_nonempty_text_keeps_selector_styles_and_updates_content() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box + .box{width:70px}.box{color:red}");
        let leaf = element(owner.GetDocument(), "leaf");
        let text = owner.GetDocument().Node(leaf).Children()[0];
        let old = owner.GetDocument().ResolvedStyleHandle(leaf).unwrap();
        let returned = owner.GetDocumentMut().AppendText(leaf, " appended");
        assert_eq!(text, returned);
        update(&mut owner);
        assert_eq!(owner.GetDocument().Node(text).Data(), "text appended");
        assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
        assert!(owner.GetDocument().StyleState().impact.layout);
        assert!(Arc::ptr_eq(
            &old,
            &owner.GetDocument().ResolvedStyleHandle(leaf).unwrap()
        ));
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().SetTextContent(text, String::new());
        update(&mut owner);
        assert!(owner.GetDocument().StyleState().stats.resolved_nodes > 0);
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn inherited_custom_properties_remain_isolated_after_local_override() {
    crate::native_test_thread::run(|| {
        let mut owner =
            fixture("section{--size:20px}.box{width:var(--size)}span{height:var(--size)}");
        let a = element(owner.GetDocument(), "a");
        let b = element(owner.GetDocument(), "b");
        let leaf = element(owner.GetDocument(), "leaf");
        assert!(Arc::ptr_eq(
            &owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .custom_properties,
            &owner
                .GetDocument()
                .ResolvedStyleFor(b)
                .unwrap()
                .custom_properties
        ));
        attribute(&mut owner, "a", "style", "--size:45px");
        update(&mut owner);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(45.0)
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(leaf)
                .unwrap()
                .style
                .height,
            Some(45.0)
        );
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(b).unwrap().style.width,
            Some(20.0)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn character_data_keeps_textarea_default_and_style_sheet_invalidation() {
    crate::native_test_thread::run(|| {
        let mut owner = html::html_parser::ParseHTML("<html><head><style id=sheet>textarea{width:20px}</style></head><body><textarea id=control placeholder=hint>old</textarea></body></html>");
        owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(
            "textarea{width:20px}textarea:placeholder-shown{width:40px}",
        ));
        update(&mut owner);
        let control = element(owner.GetDocument(), "control");
        let text = owner.GetDocument().Node(control).Children()[0];
        owner.GetDocumentMut().AppendText(control, " suffix");
        update(&mut owner);
        assert_eq!(owner.GetDocument().ControlValue(control), "old suffix");
        assert!(owner.GetDocument().StyleState().stats.resolved_nodes > 0);
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().SetTextContent(text, String::new());
        update(&mut owner);
        assert_eq!(owner.GetDocument().ControlValue(control), "");
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(control)
                .unwrap()
                .style
                .width,
            Some(40.0)
        );
        assert_full_recalc_equal(&mut owner);
        let sheet = element(owner.GetDocument(), "sheet");
        let sheet_id = owner.GetDocument().Node(sheet).Id();
        owner
            .GetDocumentMut()
            .StyleStateMut()
            .dirty_style_elements
            .clear();
        owner
            .GetDocumentMut()
            .AppendText(sheet, "textarea{height:30px}");
        assert!(owner
            .GetDocument()
            .StyleState()
            .dirty_style_elements
            .contains(&sheet_id));
    });
}

#[test]
fn repeated_media_conditions_refresh_for_environment_and_cssom() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(".box{width:10px}@media screen and (min-width:600px){.box{width:80px}.box::before{content:'wide'}}@media(prefers-color-scheme:dark){.box{height:90px}}");
        let a = element(owner.GetDocument(), "a");
        let mut env = media_queries::media_values_cached::MediaValuesCachedData {
            viewport_width: 800.0,
            viewport_height: 600.0,
            small_viewport_width: 800.0,
            small_viewport_height: 600.0,
            large_viewport_width: 800.0,
            large_viewport_height: 600.0,
            dynamic_viewport_width: 800.0,
            dynamic_viewport_height: 600.0,
            preferred_color_scheme: PreferredColorScheme::kLight,
            ..Default::default()
        };
        update_with_environment(&mut owner, &env);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(80.0)
        );
        let expected = owner.GetDocument().ResolvedStyleFor(a).unwrap().clone();
        owner.GetDocumentMut().InvalidateAllStyles();
        update_with_environment(&mut owner, &env);
        assert!(owner.GetDocument().ResolvedStyleFor(a).unwrap() == &expected);
        env.viewport_width = 400.0;
        env.small_viewport_width = 400.0;
        env.large_viewport_width = 400.0;
        env.dynamic_viewport_width = 400.0;
        env.preferred_color_scheme = PreferredColorScheme::kDark;
        update_with_environment(&mut owner, &env);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(10.0)
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .style
                .height,
            Some(90.0)
        );
        owner
            .GetDocumentMut()
            .AppendStyleSheet(style::ParseCSS("@media(max-width:500px){.box{width:55px}}"));
        update_with_environment(&mut owner, &env);
        assert_eq!(
            owner.GetDocument().ResolvedStyleFor(a).unwrap().style.width,
            Some(55.0)
        );
        let expected = owner.GetDocument().ResolvedStyleFor(a).unwrap().clone();
        owner.GetDocumentMut().InvalidateAllStyles();
        update_with_environment(&mut owner, &env);
        assert!(owner.GetDocument().ResolvedStyleFor(a).unwrap() == &expected);
    });
}

#[test]
fn content_on_originating_element_keeps_pseudo_generated_content() {
    crate::native_test_thread::run(|| {
        let mut owner = fixture(
            ".box{content:'ordinary';width:23px}.box::before{content:'pseudo';height:12px}",
        );
        let a = element(owner.GetDocument(), "a");
        let resolved = owner.GetDocument().ResolvedStyleFor(a).unwrap();
        assert_eq!(resolved.style.width, Some(23.0));
        assert_eq!(resolved.before.as_ref().unwrap().text, "pseudo");
        attribute(&mut owner, "a", "style", "content:normal");
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(a)
                .unwrap()
                .before
                .as_ref()
                .unwrap()
                .text,
            "pseudo"
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn bounded_has_class_membership_structure_moves_cascade_and_pseudo_match_full_recalc() {
    crate::native_test_thread::run(|| {
        fn owner() -> DOM {
            let mut owner = html::html_parser::ParseHTML("<html><body><section id=unrelated><span id=retained>outside</span></section><section id=wrap><div id=host class=host><div id=bin><div id=x class=item></div><div id=y></div></div><div id=target class=target></div></div><div id=host2 class=host><div id=bin2></div><div id=target2 class=target></div></div></section></body></html>");
            owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS(".target{width:20px;height:10px}.host:has(.active) .target{height:42px;color:red}.host:has(.item:first-child) .target{width:11px}.host:has(.active,#x) .target{--custom:29px}.host:has(.active) .target::before{content:'active';width:var(--custom,7px)}.host + .host{padding:2px}"));
            update(&mut owner);
            owner
        }
        let mut optimized = owner();
        let mut baseline = owner();
        let outside = element(optimized.GetDocument(), "retained");
        let handle = optimized
            .GetDocument()
            .ResolvedStyleHandle(outside)
            .unwrap();
        let check = |optimized: &mut DOM, baseline: &mut DOM| {
            baseline.GetDocumentMut().InvalidateAllStyles();
            update(optimized);
            update(baseline);
            for node in 0..optimized.GetDocument().NodeCount() {
                assert!(
                    optimized.GetDocument().ResolvedStyleFor(node)
                        == baseline.GetDocument().ResolvedStyleFor(node),
                    "node {node}"
                );
            }
            assert!(
                optimized.GetDocument().StyleState().stats.resolved_nodes
                    < baseline.GetDocument().StyleState().stats.resolved_nodes
            );
            assert!(Arc::ptr_eq(
                &handle,
                &optimized
                    .GetDocument()
                    .ResolvedStyleHandle(outside)
                    .unwrap()
            ));
        };
        for (id, value) in [("x", "item active"), ("host", ""), ("host", "host")] {
            attribute(&mut optimized, id, "class", value);
            attribute(&mut baseline, id, "class", value);
            check(&mut optimized, &mut baseline);
        }
        for owner in [&mut optimized, &mut baseline] {
            let x = element(owner.GetDocument(), "x");
            let bin2 = element(owner.GetDocument(), "bin2");
            owner.GetDocumentMut().AppendChild(bin2, x);
        }
        check(&mut optimized, &mut baseline);
        let target = element(optimized.GetDocument(), "target");
        let target2 = element(optimized.GetDocument(), "target2");
        assert_eq!(
            optimized
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .style
                .height,
            Some(10.)
        );
        assert_eq!(
            optimized
                .GetDocument()
                .ResolvedStyleFor(target2)
                .unwrap()
                .style
                .height,
            Some(42.)
        );
        assert_eq!(
            optimized
                .GetDocument()
                .ResolvedStyleFor(target2)
                .unwrap()
                .before
                .as_ref()
                .unwrap()
                .text,
            "active"
        );
        for owner in [&mut optimized, &mut baseline] {
            let x = element(owner.GetDocument(), "x");
            let y = element(owner.GetDocument(), "y");
            let bin = element(owner.GetDocument(), "bin");
            owner.GetDocumentMut().AppendChild(bin, x);
            owner.GetDocumentMut().InsertBefore(bin, y, x);
        }
        check(&mut optimized, &mut baseline);
        assert_eq!(
            optimized
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .style
                .width,
            Some(20.)
        );
        for owner in [&mut optimized, &mut baseline] {
            let x = element(owner.GetDocument(), "x");
            owner.GetDocumentMut().Remove(x);
        }
        check(&mut optimized, &mut baseline);
        assert_eq!(
            optimized
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .style
                .height,
            Some(10.)
        );
    });
}

#[test]
fn bounded_has_nonlocal_and_unknown_grammar_still_recalculates_the_root() {
    crate::native_test_thread::run(|| {
        for selector in [
            ".box:has(.active) + .box",
            "body .box:has(.active)",
            ".box:has(:future(.active))",
            ".box:has(.active ~ .other)",
            ".box:has(:is(.active:has(.other)))",
            r".box:has([cl\61ss])",
        ] {
            let mut owner = fixture(&format!("{selector}{{width:88px}}.box{{width:10px}}"));
            attribute(&mut owner, "leaf", "class", "active");
            update(&mut owner);
            assert!(
                owner.GetDocument().StyleState().stats.resolved_nodes > 2,
                "{selector}"
            );
            assert_full_recalc_equal(&mut owner);
        }
    });
}

#[test]
fn bounded_has_native_control_cssom_geometry_pixels_match_full_recalc() {
    crate::native_test_thread::run(|| {
        use javascript::{
            javascript_runtime::JavaScriptRuntime,
            quickjs_javascript_runtime::QuickJsJavaScriptRuntime,
        };
        use std::{cell::RefCell, rc::Rc};
        use webapi::dom_bindings::DOMJavaScriptBindings;
        fn run(full: bool) -> (Vec<paint::paint_engine::PaintRect>, Vec<u8>, usize) {
            let document=Rc::new(RefCell::new(html::html_parser::ParseHTML("<html><body style='margin:0'><section class=host><div><span id=toggle></span></div><textarea id=q style='box-sizing:border-box;border:0;padding:0;width:80px'>abc</textarea></section><section><div>outside retained text</div></section></body></html>")));
            document.borrow_mut().GetDocumentMut().AppendStyleSheet(style::ParseCSS("textarea{height:20px}.host:has(.active) textarea{height:30px}section + section{padding:1px}"));
            let constraints = Rc::new(RefCell::new(crate::CreateBrowserConstraints(320, 200)));
            crate::style_services::ResolveLayoutStyles(
                &mut document.borrow_mut(),
                &constraints.borrow(),
            );
            let q = element(document.borrow().GetDocument(), "q");
            let qid = document.borrow().GetDocument().Node(q).Id();
            let interaction = dom::UserInteractionState::default();
            let mut layout_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
            let before = crate::LayoutPersistentDocument(
                &mut layout_engine,
                &mut document.borrow_mut(),
                &interaction,
                &constraints.borrow(),
            );
            let old_rects = paint::paint_engine::FragmentClientRects(&before, qid);
            assert_eq!(old_rects[0].height, 20.);
            document
                .borrow_mut()
                .GetDocumentMut()
                .SetControlValue(q, "abcd".into());
            attribute(&mut document.borrow_mut(), "toggle", "class", "active");
            if full {
                document.borrow_mut().GetDocumentMut().InvalidateAllStyles();
            }
            crate::style_services::ResolveLayoutStyles(
                &mut document.borrow_mut(),
                &constraints.borrow(),
            );
            let resolved = document
                .borrow()
                .GetDocument()
                .StyleState()
                .stats
                .resolved_nodes;
            let host = crate::style_services::CreateLayoutBindingsHost(
                document.clone(),
                constraints.clone(),
                Rc::new(RefCell::new(interaction.clone())),
            );
            let mutated = document.clone();
            let bindings = Rc::new(RefCell::new(DOMJavaScriptBindings::WithHost(
                document.clone(),
                Box::new(move |m| {
                    crate::dom_mutation::ApplyDOMTreeMutation(&mut mutated.borrow_mut(), m)
                }),
                host,
            )));
            let mut runtime = QuickJsJavaScriptRuntime::new();
            let realm = runtime.CreateRealm(bindings);
            let bootstrap = runtime.Evaluate(&realm, webapi::DOMBootstrapSource(), "dom-webidl");
            assert!(bootstrap.Succeeded(), "{:?}", bootstrap.exception);
            let checked=runtime.Evaluate(&realm,"const q=document.getElementById('q');if(q.value!=='abcd'||getComputedStyle(q).height!=='30px'||q.offsetHeight!==30||q.getClientRects().length!==1||q.getBoundingClientRect().height!==30)throw Error('bounded has native control CSSOM mismatch')","bounded-has-native-cssom");
            assert!(checked.Succeeded(), "{:?}", checked.exception);
            let current = crate::LayoutPersistentDocument(
                &mut layout_engine,
                &mut document.borrow_mut(),
                &interaction,
                &constraints.borrow(),
            );
            let rects = paint::paint_engine::FragmentClientRects(&current, qid);
            assert_eq!(
                paint::paint_engine::FragmentClientRects(&before, qid),
                old_rects
            );
            let pixels = raster::pure_replay::RasterizeDisplayItemList(
                &paint::paint_engine::Paint(&current),
                320,
                200,
            );
            (rects, pixels, resolved)
        }
        let (old_rects, old_pixels, old_resolved) = run(true);
        let (rects, pixels, resolved) = run(false);
        assert_eq!(rects, old_rects);
        assert_eq!(pixels, old_pixels);
        assert!(resolved < old_resolved);
    });
}

#[test]
fn parent_changed_stops_at_unchanged_children_and_matches_native_fresh_pixels() {
    crate::native_test_thread::run(|| {
        fn owner() -> DOM {
            let mut owner=html::html_parser::ParseHTML("<html><body style='margin:0'><div id=a><div id=mid class=branch><div id=leaf><span id=text>nested text</span><textarea id=q>abc</textarea></div></div></div><div id=outside>outside text</div></body></html>");
            owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#a{width:80px;height:30px;--label:'before';--tone:black}#mid{width:50%;height:inherit}#leaf{height:inherit;color:var(--tone)}#a::before{content:var(--label);width:inherit;height:2px;background:red;opacity:inherit}textarea{width:40px;height:10px;border:0;padding:0}"));
            owner
        }
        let mut optimized = owner();
        let mut optimized_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
        let constraints = crate::CreateBrowserConstraints(320, 200);
        let interaction = dom::UserInteractionState::default();
        crate::style_services::ResolveLayoutStyles(&mut optimized, &constraints);
        let outside = element(optimized.GetDocument(), "outside");
        let retained = optimized
            .GetDocument()
            .ResolvedStyleHandle(outside)
            .unwrap();
        let q = element(optimized.GetDocument(), "q");
        let qid = optimized.GetDocument().Node(q).Id();
        for (iteration, value) in [
            "width:160px",
            "width:160px;height:45px",
            "width:160px;height:45px;--tone:red;--label:'after'",
            "display:none",
            "display:contents;--tone:blue",
            "width:120px;height:25px;opacity:0.5",
            "",
        ]
        .into_iter()
        .enumerate()
        {
            let before = crate::LayoutPersistentDocument(
                &mut optimized_engine,
                &mut optimized,
                &interaction,
                &constraints,
            );
            let old_rects = paint::paint_engine::FragmentClientRects(&before, qid);
            // Independently parsed owner has no persistent native layout tree.
            let mut baseline = owner();
            let mut baseline_engine = crate::LayoutEngine::new(&crate::CreateLayoutAssembly());
            attribute(&mut optimized, "a", "style", value);
            attribute(&mut baseline, "a", "style", value);
            baseline.GetDocumentMut().InvalidateAllStyles();
            crate::style_services::ResolveLayoutStyles(&mut optimized, &constraints);
            crate::style_services::ResolveLayoutStyles(&mut baseline, &constraints);
            let resolved = optimized.GetDocument().StyleState().stats.resolved_nodes;
            for node in 0..optimized.GetDocument().NodeCount() {
                assert!(
                    optimized.GetDocument().ResolvedStyleFor(node)
                        == baseline.GetDocument().ResolvedStyleFor(node),
                    "iteration {iteration}, node {node}"
                );
            }
            if iteration == 0 {
                assert!(
                    resolved <= 2,
                    "non-inherited width change stops after the unchanged immediate child"
                );
            }
            assert!(Arc::ptr_eq(
                &retained,
                &optimized
                    .GetDocument()
                    .ResolvedStyleHandle(outside)
                    .unwrap()
            ));
            let actual = crate::LayoutPersistentDocument(
                &mut optimized_engine,
                &mut optimized,
                &interaction,
                &constraints,
            );
            let expected = crate::LayoutPersistentDocument(
                &mut baseline_engine,
                &mut baseline,
                &interaction,
                &constraints,
            );
            let expected_qid = baseline
                .GetDocument()
                .Node(element(baseline.GetDocument(), "q"))
                .Id();
            assert_eq!(
                paint::paint_engine::FragmentClientRects(&actual, qid),
                paint::paint_engine::FragmentClientRects(&expected, expected_qid)
            );
            assert_eq!(
                paint::paint_engine::FragmentClientRects(&before, qid),
                old_rects,
                "old fragment snapshot remains immutable"
            );
            assert_eq!(
                raster::pure_replay::RasterizeDisplayItemList(
                    &paint::paint_engine::Paint(&actual),
                    320,
                    200
                ),
                raster::pure_replay::RasterizeDisplayItemList(
                    &paint::paint_engine::Paint(&expected),
                    320,
                    200
                ),
                "iteration {iteration}"
            );
        }
    });
}

#[test]
fn parent_changed_explicit_variable_inherit_and_custom_override_remain_exact() {
    crate::native_test_thread::run(|| {
        let mut owner=html::html_parser::ParseHTML("<html><body><div id=a><div id=mid><div id=deep></div></div><div id=isolated><span id=isolatedleaf>text</span></div></div></body></html>");
        owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#a{width:10px;--keyword:inherit;--tone:red}#mid{width:var(--keyword,inherit)}#deep{width:inherit}#isolated{--tone:black}#isolatedleaf{color:var(--tone)}"));
        update(&mut owner);
        attribute(&mut owner, "a", "style", "width:45px");
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(element(owner.GetDocument(), "deep"))
                .unwrap()
                .style
                .width,
            Some(45.)
        );
        assert_full_recalc_equal(&mut owner);
        let leaf = element(owner.GetDocument(), "isolatedleaf");
        let handle = owner.GetDocument().ResolvedStyleHandle(leaf).unwrap();
        attribute(&mut owner, "a", "style", "width:45px;--tone:blue");
        update(&mut owner);
        assert!(
            Arc::ptr_eq(
                &handle,
                &owner.GetDocument().ResolvedStyleHandle(leaf).unwrap()
            ),
            "unchanged overridden child inputs do not force deeper descendants"
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn parent_changed_does_not_shorten_node_or_children_selector_subtrees() {
    crate::native_test_thread::run(|| {
        let mut owner=html::html_parser::ParseHTML("<html><body><div id=a><div id=mid><section id=branch class=branch><div><span id=deep>deep</span></div></section></div></div></body></html>");
        owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#deep{height:10px;width:10px}.active #deep{height:77px}#mid > .branch:first-child #deep{width:33px}"));
        update(&mut owner);
        attribute(&mut owner, "a", "class", "active");
        update(&mut owner);
        let deep = element(owner.GetDocument(), "deep");
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(deep)
                .unwrap()
                .style
                .height,
            Some(77.)
        );
        assert_full_recalc_equal(&mut owner);
        let mid = element(owner.GetDocument(), "mid");
        let branch = element(owner.GetDocument(), "branch");
        let inserted = owner
            .GetDocumentMut()
            .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "div".into());
        owner.GetDocumentMut().InsertBefore(mid, inserted, branch);
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(deep)
                .unwrap()
                .style
                .width,
            Some(10.)
        );
        assert_full_recalc_equal(&mut owner);
        owner.GetDocumentMut().Remove(inserted);
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(deep)
                .unwrap()
                .style
                .width,
            Some(33.)
        );
        assert_full_recalc_equal(&mut owner);
        attribute(&mut owner, "a", "class", "");
        update(&mut owner);
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(deep)
                .unwrap()
                .style
                .height,
            Some(10.)
        );
        assert_full_recalc_equal(&mut owner);
    });
}

#[test]
fn parent_changed_bypasses_resource_only_shortcut_when_inherited_inputs_change() {
    crate::native_test_thread::run(|| {
        let mut owner = html::html_parser::ParseHTML(
            "<html><body><div id=a><div id=mid><span id=deep>deep</span></div></div></body></html>",
        );
        owner.GetDocumentMut().AppendStyleSheet(style::ParseCSS("#a{height:20px}#mid{height:inherit;background-image:url(test:child)}#deep{color:inherit}"));
        update(&mut owner);
        attribute(&mut owner, "a", "style", "height:30px;color:red");
        owner.GetDocumentMut().SetImageResource(
            "test:child".into(),
            dom::ImageResourceMetadata {
                id: 2,
                natural_width: 20.,
                natural_height: 20.,
                resolution_scale: 1.,
            },
        );
        update(&mut owner);
        let mid = element(owner.GetDocument(), "mid");
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(mid)
                .unwrap()
                .style
                .height,
            Some(30.)
        );
        assert_eq!(
            owner
                .GetDocument()
                .ResolvedStyleFor(mid)
                .unwrap()
                .style
                .paint
                .background_images[0]
                .resource_id,
            2
        );
        assert_full_recalc_equal(&mut owner);
    });
}

use crate::{media_queries::MediaValuesCachedData, ParseCSS, ParseUserAgentCSS, StyleEngine};
use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};
use foundation::{EDisplay, TextDirection};

fn media(width: f64) -> MediaValuesCachedData {
    MediaValuesCachedData {
        media_type: foundation::String::from("screen"),
        viewport_width: width,
        viewport_height: 600.0,
        large_viewport_width: width,
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

fn width(owner: &dom::DOM, index: usize) -> f32 {
    let style = owner.GetDocument().ResolvedStyleFor(index).unwrap();
    unsafe { &*style.native_style.Get() }.Width().Pixels()
}

fn height(owner: &dom::DOM, index: usize) -> f32 {
    let style = owner.GetDocument().ResolvedStyleFor(index).unwrap();
    unsafe { &*style.native_style.Get() }.Height().Pixels()
}

fn display(owner: &dom::DOM, index: usize) -> EDisplay {
    let style = owner.GetDocument().ResolvedStyleFor(index).unwrap();
    unsafe { &*style.native_style.Get() }.Display()
}

fn direction(owner: &dom::DOM, index: usize) -> TextDirection {
    let style = owner.GetDocument().ResolvedStyleFor(index).unwrap();
    unsafe { &*style.native_style.Get() }.Direction()
}

#[test]
fn retained_directionality_drives_presentational_hints_and_flat_tree_auto() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html>\
         <div id=rtl dir=rtl><span id=inherited></span><span id=ltr dir=ltr></span></div>\
         <div id=auto dir=auto>123 אבג</div>\
         <x-host id=host dir=rtl><span id=assigned slot=named>abc</span></x-host>",
    );
    let rtl = node(&owner, "rtl");
    let inherited = node(&owner, "inherited");
    let ltr = node(&owner, "ltr");
    let auto = node(&owner, "auto");
    let host = node(&owner, "host");
    let assigned = node(&owner, "assigned");
    let (shadow_inherited, slot_ancestor, slot) = {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let shadow_inherited = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "id".into(),
                value: "shadow-inherited".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, shadow_inherited);
        let slot_ancestor = document.CreateElement(
            DOMNamespace::kHTML,
            "div".into(),
            vec![
                DOMAttribute {
                    local_name: "id".into(),
                    value: "slot-ancestor".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "dir".into(),
                    value: "auto".into(),
                    ..Default::default()
                },
            ],
        );
        let slot = document.CreateElement(
            DOMNamespace::kHTML,
            "slot".into(),
            vec![
                DOMAttribute {
                    local_name: "id".into(),
                    value: "slot".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "name".into(),
                    value: "named".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "dir".into(),
                    value: "auto".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(slot_ancestor, slot);
        document.AppendChild(shadow, slot_ancestor);
        (shadow_inherited, slot_ancestor, slot)
    };

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    for element in [rtl, inherited, host, shadow_inherited] {
        assert_eq!(
            owner.GetDocument().CachedDirectionality(element),
            TextDirection::kRtl,
            "cached direction for {}",
            owner
                .GetDocument()
                .Node(element)
                .FindAttribute("id")
                .unwrap()
                .value
        );
        assert_eq!(
            direction(&owner, element),
            TextDirection::kRtl,
            "computed direction for {}",
            owner
                .GetDocument()
                .Node(element)
                .FindAttribute("id")
                .unwrap()
                .value
        );
    }
    assert_eq!(direction(&owner, ltr), TextDirection::kLtr);
    assert_eq!(direction(&owner, auto), TextDirection::kRtl);
    // A nested element with dir=auto skips a descendant with its own valid
    // dir value, including this independently auto-directional slot.
    assert_eq!(direction(&owner, slot_ancestor), TextDirection::kLtr);
    assert_eq!(direction(&owner, slot), TextDirection::kLtr);

    owner
        .GetDocumentMut()
        .SetTextContent(auto, "plain ltr".into());
    owner
        .GetDocumentMut()
        .SetTextContent(assigned, "مرحبا".into());
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(direction(&owner, auto), TextDirection::kLtr);
    assert_eq!(direction(&owner, slot), TextDirection::kRtl);
    assert_eq!(direction(&owner, slot_ancestor), TextDirection::kLtr);
    assert_eq!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_fallbacks,
        0
    );
}

#[test]
fn update_discovers_shadow_scope_and_cascades_host_slotted_and_part_rules() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>x-host::part(label){padding-left:104px}</style>\
         <x-host id=host><span id=light class=light slot=named></span>\
         <input id=control slot=named placeholder=hint></x-host>",
    );
    let host = node(&owner, "host");
    let light = node(&owner, "light");
    let control = node(&owner, "control");
    let (shadow, inner, shadow_style) = {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let style = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let text = document.CreateText(
            ":host{width:101px}\
             ::slotted(.light){height:102px}\
             ::slotted(input)::placeholder{font-size:37px}\
             .inner{width:103px}\
             :host::part(label){margin-left:105px}"
                .into(),
        );
        document.AppendChild(style, text);
        document.AppendChild(shadow, style);
        let slot = document.CreateElement(
            DOMNamespace::kHTML,
            "slot".into(),
            vec![DOMAttribute {
                local_name: "name".into(),
                value: "named".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, slot);
        let inner = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![
                DOMAttribute {
                    local_name: "id".into(),
                    value: "inner".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "class".into(),
                    value: "inner".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "part".into(),
                    value: "label".into(),
                    ..Default::default()
                },
                // Force the compatibility-entry merge so the per-TreeScope
                // MatchResult owner is preserved when inline style is present.
                DOMAttribute {
                    local_name: "style".into(),
                    value: "color:rgb(1 2 3)".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(shadow, inner);
        (shadow, inner, style)
    };

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    let translated = engine.TranslatedRuleShadowStats();
    assert!(translated.translated_style_resolver_applied >= 4);
    assert_eq!(translated.translated_style_resolver_fallbacks, 0);

    assert_eq!(width(&owner, host), 101.0);
    assert_eq!(height(&owner, light), 102.0);
    assert_eq!(width(&owner, inner), 103.0);
    let placeholder = owner
        .GetDocument()
        .ResolvedStyleFor(control)
        .unwrap()
        .placeholder
        .as_ref()
        .unwrap();
    assert_eq!(
        unsafe { &*placeholder.native_style.Get() }
            .GetFontDescription()
            .SpecifiedSize(),
        37.0
    );
    let inner_style = owner.GetDocument().ResolvedStyleFor(inner).unwrap();
    let inner_native = unsafe { &*inner_style.native_style.Get() };
    assert_eq!(inner_native.PaddingLeft().Pixels(), 104.0);
    assert_eq!(inner_native.MarginLeft().Pixels(), 105.0);
    assert_eq!(owner.GetDocument().TreeScopeRoot(inner), shadow);
    assert_eq!(
        owner
            .GetDocument()
            .ActiveStyleSheetsForTreeScope(shadow)
            .count(),
        1
    );

    owner.GetDocumentMut().SetTextContent(
        shadow_style,
        ":host{width:121px}\
         ::slotted(.light){height:122px}\
         ::slotted(input)::placeholder{font-size:47px}\
         .inner{width:123px}\
         :host::part(label){margin-left:125px}"
            .into(),
    );
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(width(&owner, host), 121.0);
    assert_eq!(height(&owner, light), 122.0);
    assert_eq!(width(&owner, inner), 123.0);
    let placeholder = owner
        .GetDocument()
        .ResolvedStyleFor(control)
        .unwrap()
        .placeholder
        .as_ref()
        .unwrap();
    assert_eq!(
        unsafe { &*placeholder.native_style.Get() }
            .GetFontDescription()
            .SpecifiedSize(),
        47.0
    );
    let inner_style = owner.GetDocument().ResolvedStyleFor(inner).unwrap();
    assert_eq!(
        unsafe { &*inner_style.native_style.Get() }
            .MarginLeft()
            .Pixels(),
        125.0
    );
}

#[test]
fn update_forwards_part_names_through_nested_exportparts_maps() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>x-outer::part(public){width:303px}</style>\
         <x-outer id=outer></x-outer>",
    );
    let outer = node(&owner, "outer");
    let (middle, target) = {
        let document = owner.GetDocumentMut();
        let outer_shadow = document.AttachShadowRoot(outer, DOMShadowRootMode::kOpen);
        let outer_style = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        document.AppendText(outer_style, "x-middle::part(middle){height:202px}");
        document.AppendChild(outer_shadow, outer_style);
        let middle = document.CreateElement(
            DOMNamespace::kHTML,
            "x-middle".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "middle: public".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(outer_shadow, middle);

        let middle_shadow = document.AttachShadowRoot(middle, DOMShadowRootMode::kOpen);
        let middle_style = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        document.AppendText(middle_style, "x-inner::part(inner){margin-left:101px}");
        document.AppendChild(middle_shadow, middle_style);
        let inner = document.CreateElement(
            DOMNamespace::kHTML,
            "x-inner".into(),
            vec![DOMAttribute {
                local_name: "exportparts".into(),
                value: "inner: middle".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(middle_shadow, inner);

        let inner_shadow = document.AttachShadowRoot(inner, DOMShadowRootMode::kOpen);
        let target = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![
                DOMAttribute {
                    local_name: "id".into(),
                    value: "target".into(),
                    ..Default::default()
                },
                DOMAttribute {
                    local_name: "part".into(),
                    value: "inner".into(),
                    ..Default::default()
                },
            ],
        );
        document.AppendChild(inner_shadow, target);
        (middle, target)
    };

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();

    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let style = unsafe { &*style.native_style.Get() };
    assert_eq!(style.MarginLeft().Pixels(), 101.0);
    assert_eq!(style.Height().Pixels(), 202.0);
    assert_eq!(style.Width().Pixels(), 303.0);

    owner.GetDocumentMut().SetAttribute(
        middle,
        DOMAttribute {
            local_name: "exportparts".into(),
            value: "middle: private".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let style = unsafe { &*style.native_style.Get() };
    assert_eq!(style.MarginLeft().Pixels(), 101.0);
    assert_eq!(style.Height().Pixels(), 202.0);
    assert!(style.Width().IsAuto());
}

#[test]
fn ua_placeholder_tree_scope_order_prefers_outer_normal_and_inner_important() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>\
         input::placeholder{font-size:51px!important;margin-left:61px}\
         </style><x-host id=host><input id=control slot=named placeholder=hint></x-host>",
    );
    let host = node(&owner, "host");
    let control = node(&owner, "control");
    {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let style = document.CreateElementDefault(DOMNamespace::kHTML, "style".into());
        let text = document.CreateText(
            "::slotted(input)::placeholder{font-size:41px!important;margin-left:31px}".into(),
        );
        document.AppendChild(style, text);
        document.AppendChild(shadow, style);
        let slot = document.CreateElement(
            DOMNamespace::kHTML,
            "slot".into(),
            vec![DOMAttribute {
                local_name: "name".into(),
                value: "named".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, slot);
    }

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();

    let placeholder = owner
        .GetDocument()
        .ResolvedStyleFor(control)
        .unwrap()
        .placeholder
        .as_ref()
        .unwrap();
    let placeholder = unsafe { &*placeholder.native_style.Get() };
    assert_eq!(
        placeholder.MarginLeft().Pixels(),
        61.0,
        "normal declarations prefer the outer document tree scope"
    );
    assert_eq!(
        placeholder.GetFontDescription().SpecifiedSize(),
        41.0,
        "important declarations reverse tree-scope order and prefer the inner slotted scope"
    );
}

#[test]
fn text_control_placeholder_owner_tracks_visibility_and_suggested_value() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>input::placeholder{font-size:43px}</style>\
         <input id=control placeholder=hint maxlength=1>",
    );
    let control = node(&owner, "control");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();

    let placeholder = owner
        .GetDocument()
        .TextControlPlaceholderElement(control)
        .expect("text controls retain their placeholder implementation element");
    let editor = owner
        .GetDocument()
        .TextControlInnerEditorElement(control)
        .expect("text controls retain their inner editor");
    let root = owner.GetDocument().ShadowRoot(control).unwrap();
    assert_eq!(
        owner.GetDocument().Node(root).Children(),
        &[placeholder, editor]
    );
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(placeholder),
        Some("-webkit-input-placeholder")
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(placeholder)
            .FindAttribute("style")
            .unwrap()
            .value,
        "display:block!important"
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(owner.GetDocument().Node(placeholder).Children()[0])
            .Data(),
        "hint"
    );
    assert_eq!(display(&owner, placeholder), EDisplay::kBlock);
    assert_eq!(display(&owner, editor), EDisplay::kFlowRoot);
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(editor)
                .unwrap()
                .native_style
                .Get()
        }
        .Opacity(),
        1.0
    );
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(placeholder)
                .unwrap()
                .native_style
                .Get()
        }
        .GetFontDescription()
        .SpecifiedSize(),
        43.0
    );

    owner
        .GetDocumentMut()
        .SetControlValue(control, "typed".into());
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(display(&owner, placeholder), EDisplay::kNone);

    owner
        .GetDocumentMut()
        .SetSuggestedValue(control, Some("😀x".into()));
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(placeholder),
        Some("-internal-input-suggested")
    );
    assert_eq!(
        owner
            .GetDocument()
            .UAShadowPseudoElement(control, "-internal-input-suggested"),
        Some(placeholder)
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(owner.GetDocument().Node(placeholder).Children()[0])
            .Data(),
        "�"
    );
    assert_eq!(display(&owner, placeholder), EDisplay::kBlock);
    assert_eq!(owner.GetDocument().SuggestedValue(control), Some("�"));
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(editor)
                .unwrap()
                .native_style
                .Get()
        }
        .Opacity(),
        0.0
    );

    owner.GetDocumentMut().SetSuggestedValue(control, None);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(
        unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(editor)
                .unwrap()
                .native_style
                .Get()
        }
        .Opacity(),
        1.0
    );
}

#[test]
fn internal_input_suggested_rules_use_the_retained_ua_shadow_owner() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><input id=control placeholder=hint>");
    let control = node(&owner, "control");
    owner
        .GetDocumentMut()
        .SetSuggestedValue(control, Some("suggested".into()));
    let suggested = owner
        .GetDocument()
        .TextControlPlaceholderElement(control)
        .expect("suggested value creates its retained UA-shadow element");
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(suggested),
        Some("-internal-input-suggested")
    );

    let ua = ParseUserAgentCSS(
        "input::-internal-input-suggested { \
             font-size:47px; text-overflow:ellipsis; white-space:nowrap; overflow:hidden \
         }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();

    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(suggested)
            .expect("suggested UA-shadow element must be styled")
            .native_style
            .Get()
    };
    assert_eq!(native.GetFontDescription().SpecifiedSize(), 47.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner.GetDocumentMut().SetSuggestedValue(control, None);
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(suggested),
        Some("-webkit-input-placeholder")
    );
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(suggested)
            .expect("placeholder remains in the retained UA-shadow tree")
            .native_style
            .Get()
    };
    assert_ne!(native.GetFontDescription().SpecifiedSize(), 47.0);
}

#[test]
fn meter_ua_shadow_tree_routes_custom_pseudo_rules_to_retained_nodes() {
    use foundation::String;

    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><meter id=meter min=0 max=10 low=3 high=7 optimum=1 value=9></meter>",
    );
    let meter = node(&owner, "meter");
    let root = owner
        .GetDocument()
        .ShadowRoot(meter)
        .expect("meter constructor creates a UA ShadowRoot");
    let inner = owner
        .GetDocument()
        .UAShadowPseudoElement(meter, "-webkit-meter-inner-element")
        .expect("meter inner implementation element");
    let bar = owner
        .GetDocument()
        .UAShadowPseudoElement(meter, "-webkit-meter-bar")
        .expect("meter bar implementation element");
    let value = owner
        .GetDocument()
        .UAShadowPseudoElement(meter, "-webkit-meter-even-less-good-value")
        .expect("meter value implementation element");
    assert_eq!(owner.GetDocument().Node(root).Children(), &[inner]);
    assert_eq!(owner.GetDocument().Node(inner).Children(), &[bar]);
    assert_eq!(owner.GetDocument().Node(bar).Children(), &[value]);

    let ua = ParseUserAgentCSS(
        "meter::-webkit-meter-inner-element { \
             display:grid; grid-template-rows:1fr [line1] 2fr [line2] 1fr \
         } \
         meter::-webkit-meter-bar { \
             position:absolute; grid-row-start:line1; grid-row-end:line2 \
         } \
         meter::-webkit-meter-even-less-good-value { opacity:.37 } \
         meter::-webkit-meter-optimum-value { opacity:.83 }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();

    let inner_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(inner)
            .expect("meter inner style")
            .native_style
            .Get()
    };
    assert_eq!(inner_style.Display(), EDisplay::kGrid);
    let rows = unsafe { &*inner_style.SpecifiedGridTemplateRows().Get() };
    assert_eq!(rows.GetTrackList().TrackCountWithoutAutoRepeat(), 3);
    assert_eq!(rows.GetNamedGridLines()[&String::from("line1")], [1]);
    assert_eq!(rows.GetNamedGridLines()[&String::from("line2")], [2]);
    let bar_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(bar)
            .expect("meter bar style")
            .native_style
            .Get()
    };
    assert!(bar_style.GridRowStart().IsNamedGridArea());
    assert_eq!(bar_style.GridRowStart().NamedGridLine().Utf8(), "line1");
    assert!(bar_style.GridRowEnd().IsNamedGridArea());
    assert_eq!(bar_style.GridRowEnd().NamedGridLine().Utf8(), "line2");
    let value_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(value)
            .expect("meter value style")
            .native_style
            .Get()
    };
    assert_eq!(value_style.Opacity(), 0.37);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner.GetDocumentMut().SetAttribute(
        meter,
        DOMAttribute {
            local_name: "value".into(),
            value: "2".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(value),
        Some("-webkit-meter-optimum-value")
    );
    let value_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(value)
            .expect("updated meter value style")
            .native_style
            .Get()
    };
    assert_eq!(value_style.Opacity(), 0.83);
    assert_eq!(
        owner
            .GetDocument()
            .Node(value)
            .FindAttribute("style")
            .expect("meter value owns its percentage style")
            .value,
        "inline-size:20%;block-size:100%"
    );
}

#[test]
fn progress_ua_shadow_tree_tracks_indeterminate_value_and_max() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><progress id=progress></progress>");
    let progress = node(&owner, "progress");
    let root = owner
        .GetDocument()
        .ShadowRoot(progress)
        .expect("progress constructor creates a UA ShadowRoot");
    let inner = owner
        .GetDocument()
        .UAShadowPseudoElement(progress, "-webkit-progress-inner-element")
        .expect("progress inner implementation element");
    let bar = owner
        .GetDocument()
        .UAShadowPseudoElement(progress, "-webkit-progress-bar")
        .expect("progress bar implementation element");
    let value = owner
        .GetDocument()
        .UAShadowPseudoElement(progress, "-webkit-progress-value")
        .expect("progress value implementation element");
    assert_eq!(owner.GetDocument().Node(root).Children(), &[inner]);
    assert_eq!(owner.GetDocument().Node(inner).Children(), &[bar]);
    assert_eq!(owner.GetDocument().Node(bar).Children(), &[value]);
    assert!(owner.GetDocument().ShouldAppearIndeterminate(progress));
    assert_eq!(
        owner
            .GetDocument()
            .Node(value)
            .FindAttribute("style")
            .expect("indeterminate progress owns its percentage style")
            .value,
        "inline-size:-100%;block-size:100%"
    );

    let ua = ParseUserAgentCSS(
        "progress:indeterminate { opacity:.23 } \
         progress[value] { opacity:.77 } \
         progress::-webkit-progress-inner-element { display:block } \
         progress::-webkit-progress-bar { width:73px } \
         progress::-webkit-progress-value { opacity:.41 }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    let host_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(progress)
            .expect("indeterminate progress host style")
            .native_style
            .Get()
    };
    assert_eq!(host_style.Opacity(), 0.23);
    assert_eq!(display(&owner, inner), EDisplay::kBlock);
    assert_eq!(width(&owner, bar), 73.0);
    let value_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(value)
            .expect("progress value style")
            .native_style
            .Get()
    };
    assert_eq!(value_style.Opacity(), 0.41);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner.GetDocumentMut().SetAttribute(
        progress,
        DOMAttribute {
            local_name: "value".into(),
            value: "3".into(),
            ..Default::default()
        },
    );
    owner.GetDocumentMut().SetAttribute(
        progress,
        DOMAttribute {
            local_name: "max".into(),
            value: "8".into(),
            ..Default::default()
        },
    );
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    assert!(!owner.GetDocument().ShouldAppearIndeterminate(progress));
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(value),
        Some("-webkit-progress-value")
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(value)
            .FindAttribute("style")
            .expect("determinate progress owns its percentage style")
            .value,
        "inline-size:37.5%;block-size:100%"
    );
    let host_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(progress)
            .expect("determinate progress host style")
            .native_style
            .Get()
    };
    assert_eq!(host_style.Opacity(), 0.77);

    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(progress, "value");
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert!(owner.GetDocument().ShouldAppearIndeterminate(progress));
    assert_eq!(
        owner
            .GetDocument()
            .Node(value)
            .FindAttribute("style")
            .expect("indeterminate progress restores its percentage style")
            .value,
        "inline-size:-100%;block-size:100%"
    );
}

#[test]
fn search_cancel_button_is_a_retained_typed_ua_shadow_owner() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><input id=search type=search value=needle>");
    let search = node(&owner, "search");
    let root = owner
        .GetDocument()
        .ShadowRoot(search)
        .expect("search input creates a UA ShadowRoot");
    let container = owner
        .GetDocument()
        .UAShadowPseudoElement(search, "-webkit-textfield-decoration-container")
        .expect("search input retains its decoration container");
    let cancel = owner
        .GetDocument()
        .UAShadowPseudoElement(search, "-webkit-search-cancel-button")
        .expect("search input retains its cancel button");
    let editor = owner
        .GetDocument()
        .TextControlInnerEditorElement(search)
        .expect("search input retains its editor");
    assert_eq!(owner.GetDocument().Node(root).Children(), &[container]);
    let container_children = owner.GetDocument().Node(container).Children();
    assert_eq!(container_children.len(), 2);
    let viewport = container_children[0];
    assert_eq!(owner.GetDocument().Node(viewport).Children(), &[editor]);
    assert_eq!(container_children[1], cancel);
    assert_eq!(
        owner.GetDocument().SearchCancelButtonHost(cancel),
        Some(search)
    );
    assert!(owner
        .GetDocument()
        .Node(cancel)
        .FindAttribute("style")
        .is_none());

    let ua = ParseUserAgentCSS(
        "input::-webkit-textfield-decoration-container { display:flex } \
         input[type=search]::-webkit-search-cancel-button { opacity:.61; width:19px }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    assert_eq!(display(&owner, container), EDisplay::kFlex);
    assert_eq!(width(&owner, cancel), 19.0);
    let cancel_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(cancel)
            .expect("search cancel style")
            .native_style
            .Get()
    };
    assert_eq!(cancel_style.Opacity(), 0.61);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner
        .GetDocumentMut()
        .SetControlValue(search, String::new());
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    assert_eq!(
        owner
            .GetDocument()
            .Node(cancel)
            .FindAttribute("style")
            .expect("empty search suppresses its cancel button")
            .value,
        "opacity:0;pointer-events:none"
    );
    let cancel_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(cancel)
            .expect("empty search cancel style")
            .native_style
            .Get()
    };
    assert_eq!(cancel_style.Opacity(), 0.0);

    owner
        .GetDocumentMut()
        .SetControlValue(search, "again".into());
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert!(owner
        .GetDocument()
        .Node(cancel)
        .FindAttribute("style")
        .is_none());
    let cancel_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(cancel)
            .expect("restored search cancel style")
            .native_style
            .Get()
    };
    assert_eq!(cancel_style.Opacity(), 0.61);
}

#[test]
fn color_input_swatch_is_a_retained_value_owned_ua_shadow_tree() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><input id=color type=color value='rgba(12,34,56,.2)'>",
    );
    let color = node(&owner, "color");
    let root = owner
        .GetDocument()
        .ShadowRoot(color)
        .expect("color input creates a UA ShadowRoot");
    let wrapper = owner
        .GetDocument()
        .UAShadowPseudoElement(color, "-webkit-color-swatch-wrapper")
        .expect("color input retains its swatch wrapper");
    let swatch = owner
        .GetDocument()
        .UAShadowPseudoElement(color, "-webkit-color-swatch")
        .expect("color input retains its swatch");
    assert_eq!(owner.GetDocument().Node(root).Children(), &[wrapper]);
    assert_eq!(owner.GetDocument().Node(wrapper).Children(), &[swatch]);
    assert!(owner
        .GetDocument()
        .TextControlInnerEditorElement(color)
        .is_none());
    assert_eq!(owner.GetDocument().ControlValue(color), "#0c2238");
    assert_eq!(
        owner
            .GetDocument()
            .Node(swatch)
            .FindAttribute("style")
            .expect("swatch owns the sanitized color")
            .value,
        "background-color:#0c2238"
    );

    let ua = ParseUserAgentCSS(
        "input[type=color]::-webkit-color-swatch-wrapper { display:flex } \
         input[type=color]::-webkit-color-swatch { opacity:.64; width:21px }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    assert_eq!(display(&owner, wrapper), EDisplay::kFlex);
    assert_eq!(width(&owner, swatch), 21.0);
    let swatch_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(swatch)
            .expect("color swatch style")
            .native_style
            .Get()
    };
    assert_eq!(swatch_style.Opacity(), 0.64);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner
        .GetDocumentMut()
        .SetControlValue(color, "rgba(255,0,0,.2)".into());
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert_eq!(owner.GetDocument().ControlValue(color), "#ff0000");
    assert_eq!(
        owner
            .GetDocument()
            .Node(swatch)
            .FindAttribute("style")
            .expect("same swatch tracks the live color value")
            .value,
        "background-color:#ff0000"
    );
}

#[test]
fn file_input_button_and_status_are_retained_ua_shadow_owners() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><input id=file type=file value='must-not-be-a-file-name'>",
    );
    let file = node(&owner, "file");
    let root = owner
        .GetDocument()
        .ShadowRoot(file)
        .expect("file input creates a UA ShadowRoot");
    let button = owner
        .GetDocument()
        .FileUploadButtonElement(file)
        .expect("file input retains its upload button");
    let status = owner
        .GetDocument()
        .FileStatusElement(file)
        .expect("file input retains its status element");
    assert_eq!(owner.GetDocument().Node(root).Children(), &[button, status]);
    assert_eq!(
        owner.GetDocument().ShadowPseudoId(button),
        Some("-webkit-file-upload-button")
    );
    assert_eq!(owner.GetDocument().UltimateOriginatingElement(button), file);
    assert_eq!(
        owner
            .GetDocument()
            .Node(button)
            .FindAttribute("type")
            .unwrap()
            .value,
        "button"
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(button)
            .FindAttribute("value")
            .unwrap()
            .value,
        "Choose file"
    );
    let status_text = owner.GetDocument().Node(status).Children()[0];
    assert_eq!(
        owner.GetDocument().Node(status_text).Data(),
        "No file selected"
    );
    assert_eq!(owner.GetDocument().ControlValue(file), "");
    assert!(owner
        .GetDocument()
        .TextControlInnerEditorElement(file)
        .is_none());

    let ua = ParseUserAgentCSS(
        "input[type=file]::-webkit-file-upload-button { opacity:.58; width:31px } \
         input[type=file]:disabled::-webkit-file-upload-button { opacity:.27 }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(800.0), &[ua.clone()])
        .unwrap();
    assert_eq!(width(&owner, button), 31.0);
    let button_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(button)
            .expect("file upload button style")
            .native_style
            .Get()
    };
    assert_eq!(button_style.Opacity(), 0.58);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    owner.GetDocumentMut().SetAttribute(
        file,
        DOMAttribute {
            local_name: "multiple".into(),
            value: String::new(),
            ..Default::default()
        },
    );
    owner.GetDocumentMut().SetAttribute(
        file,
        DOMAttribute {
            local_name: "disabled".into(),
            value: String::new(),
            ..Default::default()
        },
    );
    assert_eq!(
        owner
            .GetDocument()
            .Node(button)
            .FindAttribute("value")
            .unwrap()
            .value,
        "Choose files"
    );
    assert!(owner
        .GetDocument()
        .Node(button)
        .FindAttribute("disabled")
        .is_some());
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    let disabled_button_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(button)
            .expect("disabled file upload button style")
            .native_style
            .Get()
    };
    assert_eq!(disabled_button_style.Opacity(), 0.27);

    owner.GetDocumentMut().SetAttribute(
        file,
        DOMAttribute {
            local_name: "type".into(),
            value: "text".into(),
            ..Default::default()
        },
    );
    assert!(owner.GetDocument().FileUploadButtonElement(file).is_none());
    assert!(owner.GetDocument().FileStatusElement(file).is_none());
    assert_eq!(owner.GetDocument().Node(button).Parent(), None);
    assert_eq!(owner.GetDocument().Node(status).Parent(), None);
}

#[test]
fn update_discovers_constructed_sheet_adopted_by_shadow_scope() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><x-host id=host></x-host>");
    let host = node(&owner, "host");
    let (shadow, inner) = {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRoot(host, DOMShadowRootMode::kOpen);
        let inner = document.CreateElement(
            DOMNamespace::kHTML,
            "span".into(),
            vec![DOMAttribute {
                local_name: "class".into(),
                value: "inner".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, inner);
        let identity = document
            .RegisterConstructedStyleSheet(ParseCSS(":host{width:111px}.inner{height:112px}"))
            .unwrap();
        document
            .SetAdoptedStyleSheets(shadow, vec![identity])
            .unwrap();
        (shadow, inner)
    };

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    let translated = engine.TranslatedRuleShadowStats();
    assert!(
        translated.translated_style_resolver_applied >= 2,
        "{translated:?}"
    );
    assert_eq!(
        translated.translated_style_resolver_fallbacks, 0,
        "{translated:?}"
    );

    assert_eq!(width(&owner, host), 111.0);
    assert_eq!(height(&owner, inner), 112.0);
    assert_eq!(
        owner
            .GetDocument()
            .ActiveStyleSheetsForTreeScope(shadow)
            .count(),
        1
    );

    owner
        .GetDocumentMut()
        .SetAdoptedStyleSheets(shadow, vec![])
        .unwrap();
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    let host_style = owner.GetDocument().ResolvedStyleFor(host).unwrap();
    let inner_style = owner.GetDocument().ResolvedStyleFor(inner).unwrap();
    assert!(unsafe { &*host_style.native_style.Get() }.Width().IsAuto());
    assert!(unsafe { &*inner_style.native_style.Get() }
        .Height()
        .IsAuto());
}

#[test]
fn translated_resolver_retains_unvisited_link_state_for_link_and_descendants() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>\
         a:link{width:31px}a:visited{width:97px}\
         a:link span{height:32px}a:visited span{height:98px}\
         </style><a id=link href='/target'><span id=child></span></a>",
    );
    let link = node(&owner, "link");
    let child = node(&owner, "child");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[]).unwrap();

    let translated = engine.TranslatedRuleShadowStats();
    assert!(
        translated.translated_style_resolver_applied >= 2,
        "{translated:?}"
    );
    assert_eq!(
        translated.translated_style_resolver_fallbacks, 0,
        "{translated:?}"
    );
    assert_eq!(width(&owner, link), 31.0);
    assert_eq!(height(&owner, child), 32.0);
}

#[test]
fn pseudo_container_query_starts_at_its_originating_element() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>\
         @container (width > 100px){\
           #target{width:99px}\
           #target::before{content:'x';font-size:43px}\
         }\
         </style><div id=target style='container-type:size'></div>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert!(engine
        .SetContainerSizes(&mut owner, [(target, 200.0, 50.0)])
        .unwrap());
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();

    assert!(unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    }
    .Width()
    .IsAuto());

    let before = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .before
        .as_ref()
        .unwrap();
    assert_eq!(
        unsafe { &*before.native_style.Get() }
            .GetFontDescription()
            .SpecifiedSize(),
        43.0
    );
}

#[test]
fn retained_pipeline_covers_selector_lists_media_and_layers() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><div id=target class='item selected'></div>");
    owner.GetDocumentMut().AppendStyleSheet(ParseCSS(
        "@layer base, override; \
         @layer base { #target, .absent { width:11px; height:12px } } \
         @media (min-width:700px) { \
           @layer override { .item, .other { width:22px } } \
         }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(width(&owner, target), 22.0);
    assert_eq!(height(&owner, target), 12.0);
    let wide = engine.TranslatedRuleShadowStats();
    assert_eq!(wide.rule_sets_built, 1, "{wide:?}");
    assert!(wide.selector_matches >= 2, "{wide:?}");
    assert!(wide.typed_declarations_used >= 2, "{wide:?}");
    assert_eq!(wide.legacy_declaration_fallbacks, 0, "{wide:?}");

    engine.Update(&mut owner, &media(640.0), &[]).unwrap();
    assert_eq!(width(&owner, target), 11.0);
    assert_eq!(height(&owner, target), 12.0);
    let narrow = engine.TranslatedRuleShadowStats();
    assert_eq!(narrow.rule_sets_built, 1, "{narrow:?}");
    assert!(narrow.selector_matches >= 1, "{narrow:?}");
    assert!(narrow.typed_declarations_used >= 1, "{narrow:?}");
    assert_eq!(narrow.legacy_declaration_fallbacks, 0, "{narrow:?}");
}

#[test]
fn translated_cascade_expands_all_through_cascade_expansion() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><x-control id=target class=item></x-control>");
    owner.GetDocumentMut().AppendStyleSheet(ParseCSS(
        ".item { display:block; width:41px } \
         .item { all:initial; height:13px }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(display(&owner, target), EDisplay::kInline);
    assert_eq!(height(&owner, target), 13.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.typed_declarations_used, 2, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 0, "{stats:?}");
}

#[test]
fn owner_replacement_transfers_a_new_typed_revision_without_reparse() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style id=sheet-owner></style><div id=target class=item></div>",
    );
    let owner_node_id = owner.GetDocument().Node(node(&owner, "sheet-owner")).Id();
    let target = node(&owner, "target");
    let mut first = ParseCSS(".item, .other { width:13px }");
    first.owner_node_id = owner_node_id;
    let identity = first.Identity();
    owner.GetDocumentMut().AppendStyleSheet(first);

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(width(&owner, target), 13.0);
    assert_eq!(engine.TypedSheetOwnershipStats(), (1, 0));

    let mut replacement = ParseCSS(".item, .other { width:29px }");
    replacement.owner_node_id = owner_node_id;
    owner.GetDocumentMut().AppendStyleSheet(replacement);
    let retained = owner.GetDocument().ActiveStyleSheets().next().unwrap();
    assert_eq!(retained.Identity(), identity);
    assert_eq!(retained.Revision(), 2);

    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(width(&owner, target), 29.0);
    assert_eq!(engine.TypedSheetOwnershipStats(), (1, 0));
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.rule_sets_built, 1, "{stats:?}");
    assert!(stats.selector_matches >= 1, "{stats:?}");
    assert!(stats.typed_declarations_used >= 1, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 0, "{stats:?}");
}

#[test]
fn unsupported_typed_declaration_falls_back_at_the_rule_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target class=item></div>");
    // `repeat()` is deliberately unsupported by the translated property
    // parser today. The diagnostic is scoped to this projected rule, so the
    // whole declaration block must take the compatibility parser while the
    // retained RuleSet still owns candidate selection and selector matching.
    owner.GetDocumentMut().AppendStyleSheet(ParseCSS(
        ".item { width:37px; grid-template-columns:repeat(2, 1fr) }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[]).unwrap();
    assert_eq!(width(&owner, target), 37.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.rule_sets_built, 1, "{stats:?}");
    assert!(stats.selector_matches >= 1, "{stats:?}");
    assert_eq!(stats.typed_declarations_used, 0, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 1, "{stats:?}");
}

#[test]
fn user_agent_mode_typed_owner_feeds_the_ua_cascade() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><x-control id=target></x-control>");
    let target = node(&owner, "target");
    let ua = ParseUserAgentCSS("x-control { display:block; width:17px }");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    assert_eq!(display(&owner, target), EDisplay::kBlock);
    assert_eq!(width(&owner, target), 17.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.rule_sets_built, 1, "{stats:?}");
    assert!(stats.rule_sets_consulted >= 1, "{stats:?}");
    assert!(stats.authoritative_rules >= 1, "{stats:?}");
    assert!(stats.selector_matches >= 1, "{stats:?}");
    assert!(stats.typed_declarations_used >= 1, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 0, "{stats:?}");
}

#[test]
fn author_mode_owner_passed_as_ua_uses_the_mode_mismatch_fallback() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><x-control id=target></x-control>");
    let target = node(&owner, "target");
    let ua_with_author_mode = ParseCSS("x-control { display:block; width:19px }");
    let mut engine = StyleEngine::new(&owner);

    engine
        .Update(&mut owner, &media(800.0), &[ua_with_author_mode])
        .unwrap();
    assert_eq!(display(&owner, target), EDisplay::kBlock);
    assert_eq!(width(&owner, target), 19.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.rule_sets_consulted, 0, "{stats:?}");
    assert_eq!(stats.authoritative_rules, 0, "{stats:?}");
    assert_eq!(stats.selector_matches, 0, "{stats:?}");
    assert_eq!(stats.typed_declarations_used, 0, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 1, "{stats:?}");
}

#[test]
fn user_agent_pseudo_rules_use_translated_context_aware_collector() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><x-control id=target></x-control>");
    let target = node(&owner, "target");
    let ua = ParseUserAgentCSS(
        "x-control { display:block } x-control::before { content:'ua'; display:block }",
    );
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(
        style.before.as_ref().map(|pseudo| pseudo.text.as_str()),
        Some("ua")
    );
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.rule_sets_consulted >= 1, "{stats:?}");
    assert!(stats.selector_matches >= 1, "{stats:?}");
    assert_eq!(stats.skipped_pseudo, 0, "{stats:?}");
    assert_eq!(stats.legacy_declaration_fallbacks, 0, "{stats:?}");
    assert_eq!(
        stats.translated_style_cascade_pseudo_attempts, 1,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_pseudo_applied, 1,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_pseudo_fallbacks, 0,
        "{stats:?}"
    );
}

#[test]
fn retained_nested_declarations_keep_ua_and_author_cascade_owners() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><button id=target style='color:white'>Search</button>",
    );
    let ua = ParseUserAgentCSS(
        "button {color:red; @supports not blink-feature(AppearanceBase) {text-align:center}}",
    );
    owner.GetDocumentMut().AppendStyleSheet(ParseCSS(
        "button {width:90px; @media screen {width:108px} height:44px}",
    ));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(800.0), &[ua]).unwrap();
    let index = node(&owner, "target");
    let resolved = owner.GetDocument().ResolvedStyleFor(index).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    assert_eq!(native.GetTextAlign(), foundation::ETextAlign::kCenter);
    assert_eq!(native.Width().Pixels(), 108.0);
    assert_eq!(native.Height().Pixels(), 44.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 3, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
}

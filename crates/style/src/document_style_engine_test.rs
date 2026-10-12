use crate::{media_queries::MediaValuesCachedData, StyleEngine};
use foundation::{Color, EDisplay, EOverflow, EPosition};
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
    let d = owner.GetDocument();
    (0..d.NodeCount())
        .find(|&n| d.Node(n).FindAttribute("id").is_some_and(|a| a.value == id))
        .unwrap()
}
fn font_size(owner: &dom::DOM, index: usize) -> f32 {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(index)
            .expect("element must have resolved style")
            .native_style
            .Get()
    }
    .GetFontDescription()
    .SpecifiedSize()
}

#[test]
fn document_transition_samples_reach_translated_cascade_with_transition_priority() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><div id=target style='--size:11px;width:var(--size)'></div>",
    );
    let target = node(&owner, "target");
    let target_id = owner.GetDocument().Node(target).Id();
    owner.GetDocumentMut().SetAnimationStyle(
        target_id,
        1,
        crate::ParseCSSDeclarationList("--size:37px"),
    );
    owner.GetDocumentMut().SetTransitionStyle(
        target_id,
        2,
        crate::ParseCSSDeclarationList("--size:59px"),
    );

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let transitioned = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(transitioned.Width().Pixels(), 59.0);
    assert_eq!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_fallbacks,
        0
    );

    owner
        .GetDocumentMut()
        .SetTransitionStyle(target_id, 2, Vec::new());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let animated = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(animated.Width().Pixels(), 37.0);
}

#[test]
fn html_quirks_ua_owner_is_selected_only_in_quirks_mode() {
    let _heap = foundation::LayoutHeapScope::new();
    let html = crate::ParseUserAgentCSSWithRole("", cssom::CSSUserAgentStyleSheetRole::kHTML);
    let quirks = crate::ParseUserAgentCSSWithRole(
        "input:not([type=image i]), textarea { box-sizing: border-box; }",
        cssom::CSSUserAgentStyleSheetRole::kHTMLQuirks,
    );
    let sheets = [html, quirks];

    let mut quirks_owner = html::html_parser::ParseHTML("<input id=target>");
    assert_eq!(
        quirks_owner.GetDocument().GetCompatibilityMode(),
        dom::persistent_document::CompatibilityMode::kQuirks
    );
    let quirks_target = node(&quirks_owner, "target");
    let mut quirks_engine = StyleEngine::new(&quirks_owner);
    quirks_engine
        .Update(&mut quirks_owner, &media(), &sheets)
        .unwrap();
    let quirks_style = unsafe {
        &*quirks_owner
            .GetDocument()
            .ResolvedStyleFor(quirks_target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(quirks_style.BoxSizing(), foundation::EBoxSizing::kBorderBox);
    assert_eq!(
        quirks_engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_fallbacks,
        0
    );

    let mut standards_owner = html::html_parser::ParseHTML("<!doctype html><input id=target>");
    let standards_target = node(&standards_owner, "target");
    let mut standards_engine = StyleEngine::new(&standards_owner);
    standards_engine
        .Update(&mut standards_owner, &media(), &sheets)
        .unwrap();
    let standards_style = unsafe {
        &*standards_owner
            .GetDocument()
            .ResolvedStyleFor(standards_target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(
        standards_style.BoxSizing(),
        foundation::EBoxSizing::kContentBox
    );
}

#[test]
fn forced_colors_ua_owner_uses_dedicated_medium_and_active_environment() {
    let _heap = foundation::LayoutHeapScope::new();
    let sheets = [
        crate::ParseUserAgentCSSWithRole("", cssom::CSSUserAgentStyleSheetRole::kHTML),
        crate::ParseUserAgentCSSWithRole(
            "@media ua-forced-colors { input { display: none; } }",
            cssom::CSSUserAgentStyleSheetRole::kForcedColors,
        ),
    ];
    let mut owner = html::html_parser::ParseHTML("<!doctype html><input id=target>");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);

    let ordinary_media = media();
    engine.Update(&mut owner, &ordinary_media, &sheets).unwrap();
    let ordinary = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_ne!(ordinary.Display(), EDisplay::kNone);

    let mut forced_media = ordinary_media;
    forced_media.forced_colors = crate::media_queries::forced_colors::ForcedColors::kActive;
    owner.GetDocumentMut().StyleStateMut().all_dirty = true;
    engine.Update(&mut owner, &forced_media, &sheets).unwrap();
    let forced = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(forced.Display(), EDisplay::kNone);
    assert_eq!(
        engine
            .TranslatedRuleShadowStats()
            .translated_style_resolver_fallbacks,
        0
    );
}

#[test]
fn unvisited_link_keeps_visited_color_in_its_private_slot() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><a id=target href='https://example.test/'>target</a>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "a { color:#005599 } :visited { color:#660099 }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(
        style.InsideLink(),
        foundation::EInsideLink::kInsideUnvisitedLink
    );
    assert_eq!(
        style.GetCurrentColor(None),
        Color::FromRGB(0x00, 0x55, 0x99)
    );
    assert_eq!(
        style.GetInternalVisitedCurrentColor(None),
        Color::FromRGB(0x66, 0x00, 0x99)
    );
}

#[test]
fn named_custom_highlight_metadata_survives_translated_update() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><section id=parent><p id=target>target</p></section>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#parent::highlight(mark) { color: red } \
         #target::highlight(mark) { background-color: blue }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    let names = style.CustomHighlightNames();
    assert!(!names.is_null());
    let mark = foundation::AtomicString::from_str("mark");
    assert!(unsafe { &*names }.contains(&mark));
    let highlight = style.HighlightData().CustomHighlight(&mark);
    assert!(!highlight.is_null());
    let highlight = unsafe { &*highlight };
    assert_eq!(
        highlight.StyleType(),
        layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdHighlight
    );
    assert_eq!(highlight.GetCurrentColor(None), Color::FromRGB(0xff, 0, 0));
}

#[test]
fn highlight_cache_survives_atomic_ordinary_fallback() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><p id=target>target</p>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target:defined { color: blue } \
         #target::highlight(mark) { color: red }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert!(
        stats.translated_style_resolver_fallbacks > 0,
        "the unsupported :defined owner must force the ordinary atomic fallback: {stats:?}"
    );
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    let mark = foundation::AtomicString::from_str("mark");
    assert!(style.HasPseudoElementStyle(
        layoutng_style::style::computed_style_constants::PseudoId::kPseudoIdHighlight
    ));
    let highlight = style.HighlightData().CustomHighlight(&mark);
    assert!(!highlight.is_null());
    assert_eq!(
        unsafe { &*highlight }.GetCurrentColor(None),
        Color::FromRGB(0xff, 0, 0)
    );
}

#[test]
fn webkit_any_selector_stays_on_the_authoritative_translated_path() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><p id=target class=hit>target</p><a id=link href=#>link</a>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target:-webkit-any(.hit, .miss) { width: 41px } \
         #link:is(:-webkit-any-link, .miss) { height: 43px }",
    ));
    let target = node(&owner, "target");
    let link = node(&owner, "link");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Width().Pixels(), 41.0);
    let link_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(link)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(link_style.Height().Pixels(), 43.0);
}

#[test]
fn internal_text_field_uses_typed_input_type_and_invalidates_on_type_change() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><input id=target type=EMAIL>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target:-internal-text-field { opacity: .4 }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let first = engine.TranslatedRuleShadowStats();
    assert_eq!(first.skipped_unsupported_selector, 0, "{first:?}");
    assert_eq!(first.translated_style_resolver_fallbacks, 0, "{first:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Opacity(), 0.4);

    // HTML input types are ASCII-case-insensitive enumerated values. Changing
    // the typed owner also invalidates pseudo-class matching for this node.
    set_attribute(&mut owner, target, "type", "checkbox");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Opacity(), 1.0);
}

#[test]
fn form_state_pseudos_consume_typed_capabilities_and_live_value_state() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><input id=hidden type=HIDDEN required>\
         <input id=date type=DATE><input id=placeholder placeholder=''>\
         <fieldset id=fieldset></fieldset>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#hidden:optional { opacity:.2 } #hidden:required { opacity:.8 } \
         #date:read-write { opacity:.3 } #date:read-only { opacity:.6 } \
         #placeholder:placeholder-shown { opacity:.4 } \
         #fieldset:optional { opacity:.7 }",
    ));
    let hidden = node(&owner, "hidden");
    let date = node(&owner, "date");
    let placeholder = node(&owner, "placeholder");
    let fieldset = node(&owner, "fieldset");
    let opacity = |owner: &dom::DOM, element| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get())
            .Opacity()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let first = engine.TranslatedRuleShadowStats();
    assert_eq!(first.skipped_unsupported_selector, 0, "{first:?}");
    assert_eq!(first.translated_style_resolver_fallbacks, 0, "{first:?}");
    assert_eq!(opacity(&owner, hidden), 0.2);
    assert_eq!(opacity(&owner, date), 0.3);
    assert_eq!(opacity(&owner, placeholder), 0.4);
    assert_eq!(opacity(&owner, fieldset), 1.0);

    set_attribute(&mut owner, hidden, "type", "text");
    set_attribute(&mut owner, date, "readonly", "");
    owner
        .GetDocumentMut()
        .SetControlValue(placeholder, "entered".into());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, hidden), 0.8);
    assert_eq!(opacity(&owner, date), 0.6);
    assert_eq!(opacity(&owner, placeholder), 1.0);
}

#[test]
fn disabled_form_owner_propagates_through_fieldset_legend_and_select_groups() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><fieldset id=fieldset disabled>\
           <legend><input id=legend-exempt></legend>\
           <legend><input id=second-legend></legend>\
           <input id=blocked>\
           <select id=nested-select><optgroup id=group><option id=option>one</option></optgroup></select>\
         </fieldset>\
         <select id=outer-select disabled><optgroup><option id=outer-option>two</option></optgroup></select>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#fieldset:disabled{opacity:.11} #legend-exempt:enabled{opacity:.22} \
         #second-legend:disabled{opacity:.33} #blocked:disabled{opacity:.44} \
         #group:disabled{opacity:.55} #option:disabled{opacity:.66} \
         #outer-option:disabled{opacity:.77} #blocked:enabled{opacity:.84} \
         #group:enabled{opacity:.85} #option:enabled{opacity:.86} \
         #outer-option:enabled{opacity:.87}",
    ));
    let fieldset = node(&owner, "fieldset");
    let exempt = node(&owner, "legend-exempt");
    let second = node(&owner, "second-legend");
    let blocked = node(&owner, "blocked");
    let group = node(&owner, "group");
    let option = node(&owner, "option");
    let outer_select = node(&owner, "outer-select");
    let outer_option = node(&owner, "outer-option");
    let opacity = |owner: &dom::DOM, element| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get())
            .Opacity()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    assert_eq!(opacity(&owner, fieldset), 0.11);
    assert_eq!(opacity(&owner, exempt), 0.22);
    assert_eq!(opacity(&owner, second), 0.33);
    assert_eq!(opacity(&owner, blocked), 0.44);
    assert_eq!(opacity(&owner, group), 0.55);
    assert_eq!(opacity(&owner, option), 0.66);
    assert_eq!(opacity(&owner, outer_option), 0.77);

    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(fieldset, "disabled");
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(outer_select, "disabled");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, blocked), 0.84);
    assert_eq!(opacity(&owner, group), 0.85);
    assert_eq!(opacity(&owner, option), 0.86);
    assert_eq!(opacity(&owner, outer_option), 0.87);

    set_attribute(&mut owner, group, "disabled", "");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, group), 0.55);
    assert_eq!(opacity(&owner, option), 0.66);
}

#[test]
fn ua_list_box_and_datalist_pseudos_use_dom_owned_state_and_invalidation() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html>\
         <select id=list multiple><option>one</option></select>\
         <select id=menu multiple size=1><option>two</option></select>\
         <input id=range type=range list=choices>\
         <input id=password type=password list=choices>\
         <datalist id=choices><option value=one></datalist>",
    );
    let ua = crate::ParseUserAgentCSS(
        "select:-internal-list-box { opacity:.21 } \
         input:-internal-has-datalist { opacity:.31 }",
    );
    let list = node(&owner, "list");
    let menu = node(&owner, "menu");
    let range = node(&owner, "range");
    let password = node(&owner, "password");
    let datalist = node(&owner, "choices");
    let opacity = |owner: &dom::DOM, element| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get())
            .Opacity()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    let first = engine.TranslatedRuleShadowStats();
    assert_eq!(first.skipped_unsupported_selector, 0, "{first:?}");
    assert_eq!(first.translated_style_resolver_fallbacks, 0, "{first:?}");
    assert_eq!(opacity(&owner, list), 0.21);
    assert_eq!(opacity(&owner, menu), 1.0);
    assert_eq!(opacity(&owner, range), 0.31);
    assert_eq!(opacity(&owner, password), 1.0);

    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(list, "multiple");
    set_attribute(&mut owner, menu, "size", "2");
    set_attribute(&mut owner, datalist, "id", "renamed");
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner, list), 1.0);
    assert_eq!(opacity(&owner, menu), 0.21);
    assert_eq!(opacity(&owner, range), 1.0);

    set_attribute(&mut owner, range, "list", "renamed");
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner, range), 0.31);
}

#[test]
fn autofill_pseudos_use_live_form_control_state_and_suggested_value_producer() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><input id=field>");
    let ua = crate::ParseUserAgentCSS(
        "#field:autofill { opacity:.11 } \
         #field:-webkit-autofill { height:12px } \
         #field:-internal-autofill-previewed { width:21px } \
         #field:-internal-autofill-selected { width:31px }",
    );
    let field = node(&owner, "field");
    let opacity = |owner: &dom::DOM| unsafe { &*native_pointer(owner, field) }.Opacity();
    let width = |owner: &dom::DOM| unsafe { &*native_pointer(owner, field) }.Width();
    let height = |owner: &dom::DOM| unsafe { &*native_pointer(owner, field) }.Height();
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 1.0);
    assert!(width(&owner).IsAuto());

    // HTMLInputElement::SetSuggestedValue is the source producer of
    // WebAutofillState::kPreviewed at the fixed Chromium revision.
    owner
        .GetDocumentMut()
        .SetSuggestedValue(field, Some("preview".into()));
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    let previewed = engine.TranslatedRuleShadowStats();
    assert_eq!(previewed.skipped_unsupported_selector, 0, "{previewed:?}");
    assert_eq!(
        previewed.translated_style_resolver_fallbacks, 0,
        "{previewed:?}"
    );
    assert_eq!(opacity(&owner), 0.11);
    assert_eq!(width(&owner).Pixels(), 21.0);
    assert_eq!(height(&owner).Pixels(), 12.0);

    owner.GetDocumentMut().SetAutofillValue(
        field,
        "filled".into(),
        dom::DOMAutofillState::kAutofilled,
    );
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 0.11);
    assert_eq!(width(&owner).Pixels(), 31.0);
    assert_eq!(height(&owner).Pixels(), 12.0);

    // Ordinary value assignment uses the source default kNotFilled state and
    // invalidates all four pseudo classes through the one state owner.
    owner
        .GetDocumentMut()
        .SetControlValue(field, "script-value".into());
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner), 1.0);
    assert!(width(&owner).IsAuto());
    assert!(height(&owner).IsAuto());
}

#[test]
fn select_suggested_option_retains_identity_and_clears_on_option_lifecycle() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><select id=field><option id=one value=one>One</option>\
         <option id=two value=two>Two</option></select>",
    );
    let ua = crate::ParseUserAgentCSS(
        "#field:autofill { height:11px } \
         #field:-webkit-autofill { min-width:12px } \
         #field:-internal-autofill-previewed { opacity:.21 } \
         #field:-internal-autofill-selected { opacity:.31 }",
    );
    let field = node(&owner, "field");
    let one = node(&owner, "one");
    let two = node(&owner, "two");
    let opacity = |owner: &dom::DOM| unsafe { &*native_pointer(owner, field) }.Opacity();
    let mut engine = StyleEngine::new(&owner);

    owner
        .GetDocumentMut()
        .SetSuggestedValue(field, Some("two".into()));
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    let previewed = engine.TranslatedRuleShadowStats();
    assert_eq!(previewed.skipped_unsupported_selector, 0, "{previewed:?}");
    assert_eq!(
        previewed.translated_style_resolver_fallbacks, 0,
        "{previewed:?}"
    );
    assert_eq!(owner.GetDocument().SelectSuggestedOption(field), Some(two));
    assert_eq!(owner.GetDocument().SuggestedValue(field), Some("two"));
    assert_eq!(opacity(&owner), 0.21);

    // The owner is the option, so changing its value updates SuggestedValue
    // without losing preview state.
    set_attribute(&mut owner, two, "value", "renamed");
    assert_eq!(owner.GetDocument().SelectSuggestedOption(field), Some(two));
    assert_eq!(owner.GetDocument().SuggestedValue(field), Some("renamed"));
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 0.21);

    // OptionRemoved compares the retained pointer, clears kPreviewed, and
    // publishes a style invalidation on the owning select.
    owner.GetDocumentMut().Remove(two);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(owner.GetDocument().SelectSuggestedOption(field), None);
    assert_eq!(opacity(&owner), 1.0);

    owner
        .GetDocumentMut()
        .SetSuggestedValue(field, Some("one".into()));
    owner.GetDocumentMut().SetControlValue(field, "one".into());
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(owner.GetDocument().SelectSuggestedOption(field), None);
    assert_eq!(opacity(&owner), 1.0);

    owner.GetDocumentMut().SetAutofillValue(
        field,
        "one".into(),
        dom::DOMAutofillState::kAutofilled,
    );
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 0.31);

    // Option selection uses SelectOption's default kNotFilled state.
    set_attribute(&mut owner, one, "selected", "");
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner), 1.0);
}

#[test]
fn default_pseudo_uses_content_defaults_and_tree_order_default_button_owner() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><form id=form><input id=check type=checkbox checked>\
         <select><option id=option selected>one</option></select>\
         <input id=first type=submit><button id=second>submit</button></form>",
    );
    let ua = crate::ParseUserAgentCSS(
        "#check:default { opacity:.17 } #option:default { width:13px } \
         #first:default { height:19px } #second:default { height:23px }",
    );
    let check = node(&owner, "check");
    let option = node(&owner, "option");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    // :default for checkables observes the content attribute rather than the
    // live checkedness owner consumed by :checked.
    owner.GetDocumentMut().SetControlChecked(check, false);
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    let first_update = engine.TranslatedRuleShadowStats();
    assert_eq!(
        first_update.skipped_unsupported_selector, 0,
        "{first_update:?}"
    );
    assert_eq!(
        first_update.translated_style_resolver_fallbacks, 0,
        "{first_update:?}"
    );
    assert_eq!(unsafe { &*native_pointer(&owner, check) }.Opacity(), 0.17);
    assert_eq!(
        unsafe { &*native_pointer(&owner, option) }.Width().Pixels(),
        13.0
    );
    assert_eq!(
        unsafe { &*native_pointer(&owner, first) }.Height().Pixels(),
        19.0
    );
    assert!(unsafe { &*native_pointer(&owner, second) }
        .Height()
        .IsAuto());

    // Changing the first submit control's type moves :default to the next
    // successful submit button without mutating that button itself.
    set_attribute(&mut owner, first, "type", "button");
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert!(unsafe { &*native_pointer(&owner, first) }.Height().IsAuto());
    assert_eq!(
        unsafe { &*native_pointer(&owner, second) }
            .Height()
            .Pixels(),
        23.0
    );
}

#[test]
fn checked_option_uses_live_selectedness_dirty_state_and_select_lifecycle() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><select id=select multiple>\
         <option id=one value=one selected>one</option>\
         <option id=two value=two selected>two</option>\
         <option id=three value=three>three</option></select>",
    );
    let ua = crate::ParseUserAgentCSS(
        "#one:checked { opacity:.11 } #two:checked { opacity:.22 } \
         #three:checked { opacity:.33 }",
    );
    let select = node(&owner, "select");
    let one = node(&owner, "one");
    let two = node(&owner, "two");
    let three = node(&owner, "three");
    let opacity = |owner: &dom::DOM, option| unsafe { (&*native_pointer(owner, option)).Opacity() };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    let initial = engine.TranslatedRuleShadowStats();
    assert_eq!(initial.skipped_unsupported_selector, 0, "{initial:?}");
    assert_eq!(
        initial.translated_style_resolver_fallbacks, 0,
        "{initial:?}"
    );
    assert_eq!(opacity(&owner, one), 0.11);
    assert_eq!(opacity(&owner, two), 0.22);
    assert_eq!(opacity(&owner, three), 1.0);

    // IDL selectedness becomes dirty. Removing defaultSelected must not
    // overwrite the live false state.
    owner.GetDocumentMut().SetOptionSelected(one, false);
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(one, "selected");
    owner.GetDocumentMut().SetOptionSelected(three, true);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert!(owner.GetDocument().OptionSelectionIsDirty(one));
    assert_eq!(opacity(&owner, one), 1.0);
    assert_eq!(opacity(&owner, two), 0.22);
    assert_eq!(opacity(&owner, three), 0.33);

    // Switching to single select preserves the first selected option and
    // clears later live selectedness without clearing its dirty bit.
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(select, "multiple");
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner, two), 0.22);
    assert_eq!(opacity(&owner, three), 1.0);
    assert!(owner.GetDocument().OptionSelectionIsDirty(three));

    // Removing the selected option runs the single-select reset algorithm;
    // the first enabled option becomes selected even though it is dirty.
    owner.GetDocumentMut().Remove(two);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner, one), 0.11);
    assert_eq!(opacity(&owner, three), 1.0);

    // Setting select.value uses SelectOptionByValue: it selects the matching
    // option, makes it dirty, and deselects the previous option.
    owner
        .GetDocumentMut()
        .SetControlValue(select, "three".into());
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner, one), 1.0);
    assert_eq!(opacity(&owner, three), 0.33);
    assert_eq!(owner.GetDocument().ControlValue(select), "three");
}

#[test]
fn internal_multi_select_focus_uses_non_contiguous_interaction_owner() {
    use crate::selector_checker::SelectorCheckerBackend;
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><select id=list multiple size=4>\
         <option id=first selected>one</option><option id=second>two</option></select>",
    );
    let ua = crate::ParseUserAgentCSS(
        "select:-internal-list-box:focus-visible option:-internal-multi-select-focus { opacity:.41 }",
    );
    let list = node(&owner, "list");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let list_id = owner.GetDocument().Node(list).Id();
    let first_id = owner.GetDocument().Node(first).Id();
    let second_id = owner.GetDocument().Node(second).Id();
    let opacity =
        |owner: &dom::DOM, element| unsafe { (&*native_pointer(owner, element)).Opacity() };
    let mut engine = StyleEngine::new(&owner);
    let second_active = dom::UserInteractionState {
        focused_node_id: Some(list_id),
        focus_visible_node_id: Some(list_id),
        multi_select_focused_option_id: Some(second_id),
        ..Default::default()
    };
    let mut selectors = crate::persistent_selector::PersistentSelectorService::default();
    selectors.interaction = second_active;
    let session = selectors.StyleMatchSession(owner.GetDocument());
    let backend = session.Backend();
    assert!(backend.IsFocused(&list));
    assert!(backend.IsMultiSelectFocused(&second));
    assert!(!backend.IsMultiSelectFocused(&first));
    engine
        .SetUserInteractionState(&mut owner, second_active)
        .unwrap();
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    let active = engine.TranslatedRuleShadowStats();
    assert_eq!(active.skipped_unsupported_selector, 0, "{active:?}");
    assert_eq!(active.translated_style_resolver_fallbacks, 0, "{active:?}");
    assert!(active.translated_style_resolver_applied >= 1, "{active:?}");
    assert_eq!(opacity(&owner, first), 1.0);
    assert_eq!(opacity(&owner, second), 0.41);

    engine
        .SetUserInteractionState(
            &mut owner,
            dom::UserInteractionState {
                focused_node_id: Some(list_id),
                focus_visible_node_id: Some(list_id),
                multi_select_focused_option_id: Some(first_id),
                ..Default::default()
            },
        )
        .unwrap();
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner, first), 0.41);
    assert_eq!(opacity(&owner, second), 1.0);
}

#[test]
fn hover_and_active_follow_flat_tree_chain_label_and_top_layer_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html>\
         <div id=outer><label id=label for=control><span id=hit>hit</span></label>\
         <input id=control></div>\
         <div id=top-parent><dialog id=dialog><span id=modal-hit>modal</span></dialog></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#outer:hover{opacity:.11} #label:hover{opacity:.12} #control:hover{opacity:.13}\
         #outer:active{opacity:.21} #label:active{opacity:.22} #control:active{opacity:.23}\
         #top-parent:hover{opacity:.31} #dialog:hover{opacity:.32} #modal-hit:hover{opacity:.33}",
    ));
    let outer = node(&owner, "outer");
    let label = node(&owner, "label");
    let control = node(&owner, "control");
    let hit = node(&owner, "hit");
    let top_parent = node(&owner, "top-parent");
    let dialog = node(&owner, "dialog");
    let modal_hit = node(&owner, "modal-hit");
    owner.GetDocumentMut().ShowModalDialog(dialog);
    let hit_id = owner.GetDocument().Node(hit).Id();
    let modal_hit_id = owner.GetDocument().Node(modal_hit).Id();
    let opacity =
        |owner: &dom::DOM, element| unsafe { (&*native_pointer(owner, element)).Opacity() };
    let mut engine = StyleEngine::new(&owner);

    engine
        .SetUserInteractionState(
            &mut owner,
            dom::UserInteractionState {
                hovered_node_id: Some(hit_id),
                ..Default::default()
            },
        )
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_unsupported_selector, 0, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    assert_eq!(opacity(&owner, outer), 0.11);
    assert_eq!(opacity(&owner, label), 0.12);
    assert_eq!(opacity(&owner, control), 0.13);

    engine
        .SetUserInteractionState(
            &mut owner,
            dom::UserInteractionState {
                hovered_node_id: Some(modal_hit_id),
                ..Default::default()
            },
        )
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, top_parent), 1.0);
    assert_eq!(opacity(&owner, dialog), 0.32);
    assert_eq!(opacity(&owner, modal_hit), 0.33);

    engine
        .SetUserInteractionState(
            &mut owner,
            dom::UserInteractionState {
                pressed_node_id: Some(hit_id),
                ..Default::default()
            },
        )
        .unwrap();
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, outer), 0.21);
    assert_eq!(opacity(&owner, label), 0.22);
    assert_eq!(opacity(&owner, control), 0.23);
}

#[test]
fn indeterminate_uses_live_checkbox_radio_group_and_progress_dom_state() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html>\
         <input id=check type=checkbox>\
         <input id=radio-a type=radio name=group>\
         <input id=radio-b type=radio name=group>\
         <progress id=pending></progress>\
         <progress id=determinate value=.5></progress>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#check:indeterminate{opacity:.11} \
         #radio-a:indeterminate{opacity:.21} \
         #radio-b:indeterminate{opacity:.22} \
         #pending:indeterminate{opacity:.31} \
         #determinate:indeterminate{opacity:.32}",
    ));
    let check = node(&owner, "check");
    let radio_a = node(&owner, "radio-a");
    let radio_b = node(&owner, "radio-b");
    let pending = node(&owner, "pending");
    let determinate = node(&owner, "determinate");
    let opacity = |owner: &dom::DOM, element| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get())
            .Opacity()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let first = engine.TranslatedRuleShadowStats();
    assert_eq!(first.skipped_unsupported_selector, 0, "{first:?}");
    assert_eq!(first.translated_style_resolver_fallbacks, 0, "{first:?}");
    assert_eq!(opacity(&owner, check), 1.0);
    assert_eq!(opacity(&owner, radio_a), 0.21);
    assert_eq!(opacity(&owner, radio_b), 0.22);
    assert_eq!(opacity(&owner, pending), 0.31);
    assert_eq!(opacity(&owner, determinate), 1.0);

    owner.GetDocumentMut().SetControlIndeterminate(check, true);
    owner.GetDocumentMut().SetControlChecked(radio_a, true);
    set_attribute(&mut owner, pending, "value", ".25");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, check), 0.11);
    assert_eq!(opacity(&owner, radio_a), 1.0);
    assert_eq!(opacity(&owner, radio_b), 1.0);
    assert_eq!(opacity(&owner, pending), 1.0);

    owner.GetDocumentMut().SetControlChecked(radio_a, false);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(opacity(&owner, radio_a), 0.21);
    assert_eq!(opacity(&owner, radio_b), 0.22);
}

#[test]
fn top_layer_selectors_use_dialog_and_popover_lifecycle_state() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><dialog id=dialog open></dialog><div id=popover popover></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#dialog:modal{opacity:.11}#popover:popover-open{opacity:.21}",
    ));
    let ua = crate::ParseUserAgentCSS(
        "#dialog:-internal-dialog-in-top-layer{width:11px}\
         #popover:-internal-popover-in-top-layer{width:21px}",
    );
    let dialog = node(&owner, "dialog");
    let popover = node(&owner, "popover");
    let opacity = |owner: &dom::DOM, element| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(element)
            .unwrap()
            .native_style
            .Get())
            .Opacity()
    };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    // The content attribute opens a non-modal dialog; it does not put it in
    // the top layer and must not approximate either selector state.
    assert_eq!(opacity(&owner, dialog), 1.0);
    assert!(unsafe { &*native_pointer(&owner, dialog) }.Width().IsAuto());
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(dialog, "open");
    owner.GetDocumentMut().ShowModalDialog(dialog);
    owner.GetDocumentMut().ShowPopover(popover);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    let active = engine.TranslatedRuleShadowStats();
    assert_eq!(active.skipped_unsupported_selector, 0, "{active:?}");
    assert_eq!(active.translated_style_resolver_fallbacks, 0, "{active:?}");
    assert_eq!(opacity(&owner, dialog), 0.11);
    assert_eq!(opacity(&owner, popover), 0.21);
    assert_eq!(native_width(&owner, dialog), 11.0);
    assert_eq!(native_width(&owner, popover), 21.0);

    owner.GetDocumentMut().CloseDialog(dialog);
    owner.GetDocumentMut().HidePopover(popover);
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner, dialog), 1.0);
    assert_eq!(opacity(&owner, popover), 1.0);
    assert!(unsafe { &*native_pointer(&owner, dialog) }.Width().IsAuto());
    assert!(unsafe { &*native_pointer(&owner, popover) }
        .Width()
        .IsAuto());
}

#[test]
fn internal_is_html_uses_document_type_not_element_namespace() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><html><body><table id=table></table></body></html>",
    );
    let table = node(&owner, "table");
    let form = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "form".into());
    owner.GetDocumentMut().AppendChild(table, form);
    let ua = crate::ParseUserAgentCSS(
        ":-webkit-any(table, thead, tbody, tfoot, tr) > form:-internal-is-html {\
           display:none !important\
         }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    let html = engine.TranslatedRuleShadowStats();
    assert_eq!(html.skipped_unsupported_selector, 0, "{html:?}");
    assert_eq!(html.translated_style_resolver_fallbacks, 0, "{html:?}");
    assert_eq!(
        unsafe { &*native_pointer(&owner, form) }.Display(),
        EDisplay::kNone
    );

    // XHTML uses the HTML namespace in an XML Document. The internal pseudo
    // follows Document::IsHTMLDocument(), not the element namespace.
    let root = owner.GetDocument().Root();
    owner
        .GetDocumentMut()
        .SetDocumentType(root, dom::persistent_document::DOMDocumentType::kXML);
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_ne!(
        unsafe { &*native_pointer(&owner, form) }.Display(),
        EDisplay::kNone
    );
}

#[test]
fn internal_select_contains_input_tracks_associated_dom_descendants() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><select id=select multiple></select>");
    let select = node(&owner, "select");
    let wrapper = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "div".into());
    let input = owner.GetDocumentMut().CreateElementDefault(
        dom::persistent_document::DOMNamespace::kHTML,
        "input".into(),
    );
    owner.GetDocumentMut().AppendChild(select, wrapper);
    let ua = crate::ParseUserAgentCSS(
        "select:-internal-list-box:-internal-select-contains-input {\
           overflow:clip; display:inline-flex; flex-direction:column\
         }",
    );
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_ne!(
        unsafe { &*native_pointer(&owner, select) }.Display(),
        EDisplay::kInlineFlex
    );

    owner.GetDocumentMut().AppendChild(wrapper, input);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    let inserted = engine.TranslatedRuleShadowStats();
    assert_eq!(inserted.skipped_unsupported_selector, 0, "{inserted:?}");
    assert_eq!(
        inserted.translated_style_resolver_fallbacks, 0,
        "{inserted:?}"
    );
    assert_eq!(
        unsafe { &*native_pointer(&owner, select) }.Display(),
        EDisplay::kInlineFlex
    );

    owner.GetDocumentMut().Remove(input);
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_ne!(
        unsafe { &*native_pointer(&owner, select) }.Display(),
        EDisplay::kInlineFlex
    );
}

#[test]
fn internal_select_has_slotted_button_tracks_first_element_child() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><select id=select></select>");
    let select = node(&owner, "select");
    let button = owner.GetDocumentMut().CreateElementDefault(
        dom::persistent_document::DOMNamespace::kHTML,
        "button".into(),
    );
    let blocker = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "span".into());
    owner.GetDocumentMut().AppendChild(select, button);
    let ua = crate::ParseUserAgentCSS("select:-internal-select-has-slotted-button { opacity:.41 }");
    let opacity = |owner: &dom::DOM| unsafe { (&*native_pointer(owner, select)).Opacity() };
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();

    let initial = engine.TranslatedRuleShadowStats();
    assert_eq!(initial.skipped_unsupported_selector, 0, "{initial:?}");
    assert_eq!(
        initial.translated_style_resolver_fallbacks, 0,
        "{initial:?}"
    );
    assert_eq!(opacity(&owner), 0.41);

    owner.GetDocumentMut().InsertBefore(select, blocker, button);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 1.0);

    owner.GetDocumentMut().Remove(blocker);
    engine.Update(&mut owner, &media(), &[ua.clone()]).unwrap();
    assert_eq!(opacity(&owner), 0.41);

    set_attribute(&mut owner, select, "multiple", "");
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert_eq!(opacity(&owner), 1.0);
}

#[test]
fn style_engine_retains_each_typed_stylesheet_revision_without_reparsing() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<style id=sheet-owner></style><div class=item></div>");
    let owner_node_id = owner.GetDocument().Node(node(&owner, "sheet-owner")).Id();
    let mut first = crate::ParseCSS(".item { color: red }");
    first.owner_node_id = owner_node_id;
    let first_identity = first.Identity();
    owner.GetDocumentMut().AppendStyleSheet(first);

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(engine.TypedSheetOwnershipStats(), (1, 0));
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(engine.TypedSheetOwnershipStats(), (1, 0));

    // A same-owner, same-source parse remains a CSSOM no-op. A changed source
    // keeps the stable identity, advances the revision, and transfers the new
    // typed owner instead of asking StyleEngine to parse the projection again.
    let mut unchanged = crate::ParseCSS(".item { color: red }");
    unchanged.owner_node_id = owner_node_id;
    owner.GetDocumentMut().AppendStyleSheet(unchanged);
    let retained = owner.GetDocument().ActiveStyleSheets().next().unwrap();
    assert_eq!(
        (retained.Identity(), retained.Revision()),
        (first_identity, 1)
    );

    let mut replacement = crate::ParseCSS(".item { color: blue }");
    replacement.owner_node_id = owner_node_id;
    owner.GetDocumentMut().AppendStyleSheet(replacement);
    let retained = owner.GetDocument().ActiveStyleSheets().next().unwrap();
    assert_eq!(
        (retained.Identity(), retained.Revision()),
        (first_identity, 2)
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(engine.TypedSheetOwnershipStats(), (1, 0));
}

#[test]
fn update_reaches_retained_translated_rule_set_and_selector_checker() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><main id=parent><span id=target class='item selected'></span></main>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".item{color:red} #parent > span.item.selected{display:block} \
         .absent#never,.item{position:absolute} \
         .absent.item{width:1px} @media(min-width:700px){#target{height:3px}}",
    ));
    let target = node(&owner, "target");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let first = engine.TranslatedRuleShadowStats();
    assert_eq!(first.rule_sets_built, 1, "{first:?}");
    assert!(first.elements_probed >= 1, "{first:?}");
    assert!(first.rule_sets_consulted >= 1, "{first:?}");
    assert!(first.translated_collector_calls >= 1, "{first:?}");
    assert!(first.translated_property_sets_transferred >= 3, "{first:?}");
    assert_eq!(
        first.translated_property_sets_consumed, first.translated_property_sets_transferred,
        "collector output must be the production cascade input: {first:?}"
    );
    assert!(first.candidates_considered >= 4, "{first:?}");
    assert!(first.selector_matches >= 3, "{first:?}");
    assert!(first.authoritative_rules >= 4, "{first:?}");
    assert!(first.typed_declarations_used >= 3, "{first:?}");
    assert_eq!(first.legacy_declaration_fallbacks, 0, "{first:?}");
    assert_eq!(first.translated_style_cascade_attempts, 1, "{first:?}");
    assert_eq!(first.translated_style_cascade_applied, 1, "{first:?}");
    assert_eq!(first.translated_style_cascade_fallbacks, 0, "{first:?}");
    assert_eq!(
        first.translated_style_cascade_attempts,
        first.translated_style_cascade_applied + first.translated_style_cascade_fallbacks,
        "every production translated-cascade trial must publish or wholly fall back: {first:?}"
    );
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Display(), EDisplay::kBlock);
    assert_eq!(native.GetPosition(), EPosition::kAbsolute);

    // The typed sheet and its translated RuleSet survive a stable update.
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(engine.TranslatedRuleShadowStats().rule_sets_built, 0);

    // RuleSet media results are environment-owned. Changing the viewport
    // rebuilds that cache once and probes the real production traversal again.
    let mut narrow = media();
    narrow.viewport_width = 640.0;
    narrow.large_viewport_width = 640.0;
    engine.Update(&mut owner, &narrow, &[]).unwrap();
    let changed = engine.TranslatedRuleShadowStats();
    assert_eq!(changed.rule_sets_built, 1, "{changed:?}");
    assert!(changed.elements_probed >= 1, "{changed:?}");
    assert!(changed.rule_sets_consulted >= 1, "{changed:?}");
}

#[test]
fn update_publishes_translated_style_resolver_result_with_inline_owner() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><div id=target class=hit style='width:37px;opacity:.4'></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".hit { display:block; position:absolute; width:12px }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_attempts >= 1, "{stats:?}");
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Display(), EDisplay::kBlock);
    assert_eq!(style.GetPosition(), EPosition::kAbsolute);
    assert_eq!(style.Width().Pixels(), 37.0);
    assert!((style.Opacity() - 0.4).abs() < f32::EPSILON);
}

#[test]
fn update_publishes_translated_cascade_for_typed_numeric_math() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target { width:calc(1in + 24px); opacity:calc(.25 + .25) }",
    ));
    let target = node(&owner, "target");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Width().Pixels(), 120.0);
    assert_eq!(style.Opacity(), 0.5);
}

#[test]
fn update_resolves_pending_shorthand_from_typed_variable_data() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ":root { --edge: 3px solid rgb(10, 20, 30) } #target { border: var(--edge) }",
    ));
    let target = node(&owner, "target");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_attempts, 2, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 2, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.BorderTopWidth(), 3);
    assert_eq!(style.BorderRightWidth(), 3);
    assert_eq!(style.BorderBottomWidth(), 3);
    assert_eq!(style.BorderLeftWidth(), 3);
}

#[test]
fn document_element_uses_translated_cascade_with_initial_parent_and_root_units() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><html id=root><body><div id=child></div></body></html>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "html { color-scheme: dark; font-size: 20px; width: 2rem } \
         #child { width: 2rem }",
    ));
    let root = node(&owner, "root");
    let child = node(&owner, "child");
    let mut environment = media();
    environment.em_size = 16.0;

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &environment, &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_root_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_root_applied, 1, "{stats:?}");
    assert_eq!(
        stats.translated_style_cascade_root_fallbacks, 0,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_root_attempts,
        stats.translated_style_cascade_root_applied + stats.translated_style_cascade_root_fallbacks,
        "a root trial must publish or wholly fall back: {stats:?}"
    );

    let root_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(root)
            .unwrap()
            .native_style
            .Get()
    };
    let child_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(child)
            .unwrap()
            .native_style
            .Get()
    };
    // On the root, rem resolves against the initial font size. Descendants use
    // the computed root font. The internal root color is applied only after
    // color-scheme establishes the builder's used scheme.
    assert_eq!(root_style.Width().Pixels(), 32.0);
    assert_eq!(child_style.Width().Pixels(), 40.0);
    assert_eq!(root_style.Color().GetColor(), Color::kWhite);
}

#[test]
fn light_dark_color_uses_each_elements_computed_color_scheme() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><div id=light></div><div id=dark></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#light { color-scheme: light; color: light-dark(rgb(17, 34, 51), rgb(221, 238, 255)) } \
         #dark { color-scheme: dark; color: light-dark(rgb(17, 34, 51), rgb(221, 238, 255)) }",
    ));
    let light = node(&owner, "light");
    let dark = node(&owner, "dark");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    let style = |index| unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(index)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(
        style(light).GetCurrentColor(None),
        Color::FromRGB(17, 34, 51)
    );
    assert_eq!(
        style(dark).GetCurrentColor(None),
        Color::FromRGB(221, 238, 255)
    );
}

#[test]
fn light_dark_url_images_use_computed_scheme_and_document_resource_owner() {
    use layoutng_style::style::style_fetched_image::FetchedImage;

    let _heap = foundation::LayoutHeapScope::new();
    let light_source = "https://example.test/light.svg";
    let dark_source = "https://example.test/dark.svg";
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><div id=light></div><div id=dark></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(&format!(
        "#light {{ color-scheme:light; background-image:light-dark(url(\"{light_source}\"),url(\"{dark_source}\")) }} \
         #dark {{ color-scheme:dark; background-image:light-dark(url(\"{light_source}\"),url(\"{dark_source}\")) }}"
    )));
    for (source, id) in [(light_source, 81), (dark_source, 82)] {
        owner.GetDocumentMut().SetImageResource(
            source.into(),
            dom::ImageResourceMetadata {
                id,
                natural_width: 16.0,
                natural_height: 16.0,
                resolution_scale: 1.0,
                orientation: Default::default(),
            },
        );
    }

    let light = node(&owner, "light");
    let dark = node(&owner, "dark");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    let fetched = |index| {
        let style = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(index)
                .unwrap()
                .native_style
                .Get()
        };
        FetchedImage(unsafe { &*style.BackgroundLayers().GetImage() }).unwrap()
    };
    assert_eq!(fetched(light).SourceURL(), light_source);
    assert_eq!(fetched(light).ResourceId(), 81);
    assert_eq!(fetched(dark).SourceURL(), dark_source);
    assert_eq!(fetched(dark).ResourceId(), 82);
}

#[test]
fn color_mix_retains_current_color_and_normalized_alpha_through_native_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><main style='color:rgb(40, 80, 120)'>\
         <div id=surface></div><div id=text></div></main>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#surface { background-color: color-mix(currentColor 20%, transparent) } \
         #text { color: color-mix(currentColor 50%, transparent) }",
    ));
    let surface = node(&owner, "surface");
    let text = node(&owner, "text");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    let style = |index| unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(index)
            .unwrap()
            .native_style
            .Get()
    };
    let surface = style(surface);
    let background = surface.BackgroundColor().Resolve(
        surface.GetCurrentColor(None),
        surface.UsedColorScheme(),
        None,
    );
    let text = style(text).GetCurrentColor(None);
    for color in [background, text] {
        assert!((color.Param0() - 40.0).abs() < 0.02, "{color:?}");
        assert!((color.Param1() - 80.0).abs() < 0.02, "{color:?}");
        assert!((color.Param2() - 120.0).abs() < 0.02, "{color:?}");
    }
    assert!((background.Alpha() - 0.2).abs() < f32::EPSILON);
    assert!((text.Alpha() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn document_element_typed_inline_style_commits_in_the_same_resolver_cascade() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><html id=root style='height:23px'><body></body></html>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("html { width: 19px }"));
    let root = node(&owner, "root");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_root_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_root_applied, 1, "{stats:?}");
    assert_eq!(
        stats.translated_style_cascade_root_fallbacks, 0,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_root_attempts,
        stats.translated_style_cascade_root_applied + stats.translated_style_cascade_root_fallbacks,
        "a root trial must publish or wholly fall back: {stats:?}"
    );
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(root)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Width().Pixels(), 19.0);
    assert_eq!(style.Height().Pixels(), 23.0);
}

#[test]
fn mixed_sheet_keeps_ordinary_collector_output_as_final_cascade_input() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><main class=outer><span id=target class=item></span></main>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".item{display:block} @scope (.outer){#target{position:absolute}}",
    ));
    let target = node(&owner, "target");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_property_sets_transferred >= 1, "{stats:?}");
    assert_eq!(
        stats.translated_property_sets_consumed, stats.translated_property_sets_transferred,
        "ordinary entries from a mixed sheet must remain collector-owned: {stats:?}"
    );
    assert_eq!(
        stats.skipped_scope, 0,
        "explicit @scope is owned by the translated collector: {stats:?}"
    );
    assert_eq!(stats.translated_style_cascade_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Display(), EDisplay::kBlock);
    assert_eq!(native.GetPosition(), EPosition::kAbsolute);
}

#[test]
fn mixed_typed_and_inline_declarations_share_one_translated_cascade() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><div id=target class=item style='height:23px;position:absolute'></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".item{display:block;width:19px;opacity:.5}",
    ));
    let target = node(&owner, "target");

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_property_sets_consumed >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Display(), EDisplay::kBlock);
    assert_eq!(native.GetPosition(), EPosition::kAbsolute);
    assert_eq!(native.Width().Pixels(), 19.0);
    assert_eq!(native.Height().Pixels(), 23.0);
    assert_eq!(native.Opacity(), 0.5);
}

#[test]
fn translated_rule_path_evaluates_owner_media_outside_typed_rule_set() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><div class=item></div>");
    let mut sheet = crate::ParseCSS(".item{color:red}");
    sheet.SetLinkMetadata(0, false, false, String::new(), "print".into());
    owner.GetDocumentMut().AppendStyleSheet(sheet);

    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.rule_sets_built, 1, "{stats:?}");
    assert!(stats.elements_probed >= 1, "{stats:?}");
    assert!(stats.skipped_owner_media >= 1, "{stats:?}");
    assert_eq!(stats.rule_sets_consulted, 0, "{stats:?}");
    assert_eq!(stats.selector_matches, 0, "{stats:?}");
}

#[test]
fn resolved_background_gradient_is_retained_by_native_computed_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=target style='background:linear-gradient(to right,red,blue) left top/16px 16px no-repeat,linear-gradient(to bottom,red,blue) right bottom/20px 20px no-repeat'></div>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    let first = native.BackgroundLayers();
    let image = unsafe { first.GetImage().as_ref() }.expect("computed background image");
    assert!(image.IsGeneratedImage());
    assert!(
        layoutng_style::style::generated_gradient_image::ResolvedLinearGradient(image).is_some()
    );
    let second = unsafe { first.Next().as_ref() }.expect("second background layer");
    assert!(unsafe { second.GetImage().as_ref() }.is_some_and(|image| image.IsGeneratedImage()));
    assert_eq!(first.SizeLength().Width().Pixels(), 16.0);
    assert_eq!(second.SizeLength().Width().Pixels(), 20.0);
}

#[test]
fn url_images_cross_native_style_resource_and_paint_boundaries() {
    use layoutng_style::style::style_fetched_image::FetchedImage;

    let _heap = foundation::LayoutHeapScope::new();
    let source = "https://example.test/assets/card.png";
    let mut owner = html::html_parser::ParseHTML(&format!(
        "<div id=target style='background:center / auto 80% no-repeat url(\"{source}\") #005a9c'></div>"
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(resolved.ReferencedStyleImageSources(), [source]);
    let native = unsafe { &*resolved.native_style.Get() };
    let fetched = FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() }).unwrap();
    assert_eq!(fetched.SourceURL(), source);
    assert_eq!(fetched.ResourceId(), 0);
    let paint = layoutng::internal::boundary::native_paint_style::Export(native, None);
    assert_eq!(paint.background_images.len(), 1);
    assert_eq!(paint.background_images[0].source_url, source);
    assert_eq!(paint.background_images[0].resource_id, 0);

    owner.GetDocumentMut().SetImageResource(
        source.into(),
        dom::ImageResourceMetadata {
            id: 41,
            natural_width: 80.0,
            natural_height: 40.0,
            resolution_scale: 2.0,
            orientation: Default::default(),
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    let fetched = FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() }).unwrap();
    assert_eq!(fetched.ResourceId(), 41);
    let paint = layoutng::internal::boundary::native_paint_style::Export(native, None);
    assert_eq!(paint.background_images[0].resource_id, 41);
    assert_eq!(paint.background_images[0].source_url, source);
}

#[test]
fn image_set_reselects_for_device_scale_and_exports_per_use_resolution() {
    use layoutng_style::style::style_fetched_image::FetchedImage;

    let _heap = foundation::LayoutHeapScope::new();
    let one = "https://example.test/assets/card-1x.png";
    let two = "https://example.test/assets/card-2x.png";
    let mut owner = html::html_parser::ParseHTML(&format!(
        "<div id=target style='background-image:image-set(url(\"{one}\") 1x, url(\"{two}\") 2x)'></div>"
    ));
    let target = node(&owner, "target");
    for (source, id, width, height) in [(one, 51, 80.0, 40.0), (two, 52, 160.0, 80.0)] {
        owner.GetDocumentMut().SetImageResource(
            source.into(),
            dom::ImageResourceMetadata {
                id,
                natural_width: width,
                natural_height: height,
                resolution_scale: 1.0,
                orientation: Default::default(),
            },
        );
    }

    let mut engine = StyleEngine::new(&owner);
    let mut high_density = media();
    high_density.device_pixel_ratio = 1.5;
    engine.Update(&mut owner, &high_density, &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(resolved.ReferencedStyleImageSources(), [two]);
    let native = unsafe { &*resolved.native_style.Get() };
    let fetched = FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() }).unwrap();
    assert_eq!((fetched.ResourceId(), fetched.ResolutionScale()), (52, 2.0));
    let paint = layoutng::internal::boundary::native_paint_style::Export(native, None);
    assert_eq!(paint.background_images[0].image_resolution_scale, 2.0);

    let mut low_density = media();
    low_density.device_pixel_ratio = 1.0;
    engine.Update(&mut owner, &low_density, &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(resolved.ReferencedStyleImageSources(), [one]);
    let native = unsafe { &*resolved.native_style.Get() };
    let fetched = FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() }).unwrap();
    assert_eq!((fetched.ResourceId(), fetched.ResolutionScale()), (51, 1.0));

    // Page publishes LocalFrame's effective DPR (host DSF × layout zoom).
    low_density.layout_zoom_factor = 2.0;
    low_density.device_pixel_ratio = 2.0;
    engine.Update(&mut owner, &low_density, &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    let fetched = FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() }).unwrap();
    assert_eq!((fetched.ResourceId(), fetched.ResolutionScale()), (52, 2.0));
}

#[test]
fn list_marker_image_projects_selected_intrinsic_size_and_rebuilds_after_load() {
    let _heap = foundation::LayoutHeapScope::new();
    let one = "https://example.test/assets/marker-1x.png";
    let two = "https://example.test/assets/marker-2x.png";
    let mut owner = html::html_parser::ParseHTML(&format!(
        "<ul><li id=target style='zoom:2;list-style-type:square;list-style-image:image-set(url(\"{one}\") 1x, url(\"{two}\") 2x)'>item</li></ul>"
    ));
    let target = node(&owner, "target");
    let mut high_density = media();
    high_density.device_pixel_ratio = 1.5;
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &high_density, &[]).unwrap();
    let pending = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert!(pending
        .style
        .extended
        .as_ref()
        .unwrap()
        .list_style_image
        .is_none());

    for (source, id, width, height) in [(one, 61, 80.0, 40.0), (two, 62, 160.0, 80.0)] {
        owner.GetDocumentMut().SetImageResource(
            source.into(),
            dom::ImageResourceMetadata {
                id,
                natural_width: width,
                natural_height: height,
                resolution_scale: 1.0,
                orientation: image_resource::ImageOrientation::kRotate90,
            },
        );
    }
    engine.Update(&mut owner, &high_density, &[]).unwrap();
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let image = resolved
        .style
        .extended
        .as_ref()
        .unwrap()
        .list_style_image
        .as_ref()
        .expect("loaded list-style-image projection");
    assert_eq!((image.resource_id, image.resolution_scale), (62, 2.0));
    assert_eq!((image.natural_width, image.natural_height), (40.0, 80.0));
    assert!(image.shader.is_none());
}

#[test]
fn completed_url_image_rejects_cached_pending_style_snapshot() {
    use layoutng_style::style::style_fetched_image::FetchedImage;

    let _heap = foundation::LayoutHeapScope::new();
    let source = "https://example.test/assets/shared-card.png";
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><div id=first class=card></div><div id=second class=card></div>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS(&format!(
            ".card{{background-image:url(\"{source}\")}}"
        )));
    let cards = [node(&owner, "first"), node(&owner, "second")];
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    for &card in &cards {
        let style = owner.GetDocument().ResolvedStyleFor(card).unwrap();
        let native = unsafe { &*style.native_style.Get() };
        assert_eq!(
            FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() })
                .unwrap()
                .ResourceId(),
            0
        );
    }

    owner.GetDocumentMut().SetImageResource(
        source.into(),
        dom::ImageResourceMetadata {
            id: 91,
            natural_width: 80.0,
            natural_height: 40.0,
            resolution_scale: 1.0,
            orientation: Default::default(),
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    for card in cards {
        let style = owner.GetDocument().ResolvedStyleFor(card).unwrap();
        let native = unsafe { &*style.native_style.Get() };
        assert_eq!(
            FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() })
                .unwrap()
                .ResourceId(),
            91
        );
    }
}

#[test]
fn translated_style_cascade_uses_document_substitution_and_image_owners() {
    use layoutng_style::style::style_fetched_image::FetchedImage;

    let _heap = foundation::LayoutHeapScope::new();
    let source = "https://example.test/translated-card.png";
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><div id=target data-size=43></div>");
    let css = format!(
        "#target{{--fixture-width:37px;width:var(--fixture-width);\
         height:env(missing-size,41px);min-width:attr(data-size px);\
         background-image:url(\"{source}\")}}"
    );
    let retained = crate::production_style_sheet::ParseStyleSheet(
        &foundation::String::from(css.as_str()),
        crate::parser::css_parser_mode::CSSParserMode::kHTMLStandardMode,
    );
    assert!(
        retained.diagnostics.is_empty(),
        "{:?}",
        retained.diagnostics
    );
    assert!(retained.RetainedDeclarationsForProjectedRule(0).is_some());
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS(&css));
    owner.GetDocumentMut().SetImageResource(
        source.into(),
        dom::ImageResourceMetadata {
            id: 73,
            natural_width: 60.0,
            natural_height: 30.0,
            resolution_scale: 1.0,
            orientation: Default::default(),
        },
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_attempts, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 1, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    let style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(style.Width().Pixels(), 37.0);
    assert_eq!(style.Height().Pixels(), 41.0);
    assert_eq!(style.MinWidth().Pixels(), 43.0);
    assert!(style.HasVariableReference());
    assert!(style.HasEnv());
    assert!(style.HasAttrFunction());
    let image = FetchedImage(unsafe { &*style.BackgroundLayers().GetImage() })
        .expect("URL image must stay a fetched-image subtype");
    assert_eq!(image.SourceURL(), source);
    assert_eq!(image.ResourceId(), 73);
}

#[test]
fn pseudo_background_and_content_urls_retain_resource_dependencies() {
    use dom::persistent_document::PseudoContent;
    use layoutng_style::style::{
        content_data::ImageContentData, style_fetched_image::FetchedImage,
    };

    let _heap = foundation::LayoutHeapScope::new();
    let background = "https://example.test/w3c-white.svg";
    let content = "https://example.test/flags.png";
    let mut owner =
        html::html_parser::ParseHTML("<div class=home><a id=target></a></div><p id=language></p>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(&format!(
        ".home a:after{{content:\" \";display:block;background:center / auto 80% no-repeat url({background}) #005a9c}}#language:before{{content:url({content}) \"\\A\"}}"
    )));
    let target = node(&owner, "target");
    let language = node(&owner, "language");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );

    let after = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .after
        .as_ref()
        .unwrap();
    let native = unsafe { &*after.native_style.Get() };
    assert_eq!(
        FetchedImage(unsafe { &*native.BackgroundLayers().GetImage() })
            .unwrap()
            .SourceURL(),
        background
    );
    let before = owner
        .GetDocument()
        .ResolvedStyleFor(language)
        .unwrap()
        .before
        .as_ref()
        .unwrap();
    assert!(matches!(
        before.content.as_slice(),
        [PseudoContent::Image { resource_id: 0, .. }, PseudoContent::Text(text)] if text == "\n"
    ));
    let native = unsafe { &*before.native_style.Get() };
    let content_data = native.GetContentData().unwrap();
    assert!(unsafe { &*content_data }.IsImage());
    let image = unsafe { &*(content_data as *const ImageContentData) }.GetImage();
    assert_eq!(
        FetchedImage(unsafe { &*image }).unwrap().SourceURL(),
        content
    );
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .ReferencedStyleImageSources(),
        [background]
    );
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(language)
            .unwrap()
            .ReferencedStyleImageSources(),
        [content]
    );

    owner.GetDocumentMut().SetImageResource(
        content.into(),
        dom::ImageResourceMetadata {
            id: 42,
            natural_width: 90.0,
            natural_height: 30.0,
            resolution_scale: 2.0,
            orientation: image_resource::ImageOrientation::kRotate90,
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let before = owner
        .GetDocument()
        .ResolvedStyleFor(language)
        .unwrap()
        .before
        .as_ref()
        .unwrap();
    assert!(matches!(
        before.content.first(),
        Some(PseudoContent::Image {
            resource_id: 42,
            natural_width: Some(30.0),
            natural_height: Some(90.0),
            resolution_scale: 2.0,
            ..
        })
    ));
}

#[test]
fn generated_gradient_content_reaches_the_replaced_image_shader_boundary() {
    use dom::persistent_document::PseudoContent;
    use layoutng::internal::paint_input::PaintShaderKind;

    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target::before{content:repeating-conic-gradient(from 45deg at 25% 75%,red 0deg 30deg,blue 75%);display:block;width:80px;height:40px}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );

    let before = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .before
        .as_ref()
        .unwrap();
    let Some(PseudoContent::Image {
        resource_id,
        shader: Some(shader),
        ..
    }) = before.content.first()
    else {
        panic!("generated content image must retain a shader")
    };
    assert_eq!(*resource_id, 0);
    assert_eq!(shader.kind, PaintShaderKind::kConicGradient);
    assert_eq!(shader.rotation_angle, 45.0);
}

#[test]
fn html_type_selectors_from_ua_styles_reach_native_computed_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><head><style id=sheet>body{color:red}</style><title id=title>x</title></head><body id=body>ok</body></html>",
    );
    let sheet = node(&owner, "sheet");
    let title = node(&owner, "title");
    let body = node(&owner, "body");
    let ua = crate::ParseCSS("head,script,style,title{display:none} html,body{display:block}");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua]).unwrap();

    let display = |index| unsafe {
        (&*owner
            .GetDocument()
            .ResolvedStyleFor(index)
            .unwrap()
            .native_style
            .Get())
            .Display()
    };
    assert_eq!(display(sheet), EDisplay::kNone);
    assert_eq!(display(title), EDisplay::kNone);
    assert_eq!(display(body), EDisplay::kBlock);
    assert!(
        !owner
            .GetDocument()
            .ResolvedStyleFor(sheet)
            .unwrap()
            .generates_box
    );
    assert!(
        !owner
            .GetDocument()
            .ResolvedStyleFor(title)
            .unwrap()
            .generates_box
    );
}

#[test]
fn non_inherited_flex_direction_does_not_cross_element_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>.outer{display:flex;flex-direction:column}.inner{display:inline-flex}</style><div id=outer class=outer><a id=inner class=inner>link</a></div>",
    );
    let outer = node(&owner, "outer");
    let inner = node(&owner, "inner");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let outer = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(outer)
            .unwrap()
            .native_style
            .Get()
    };
    let inner = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(inner)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(outer.FlexDirection(), foundation::EFlexDirection::kColumn);
    assert_eq!(inner.FlexDirection(), foundation::EFlexDirection::kRow);
    assert_eq!(inner.Display(), EDisplay::kFlex);
}

#[test]
fn typed_length_division_in_zoom_reaches_native_computed_style() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=target style='zoom:min(calc(100vw / 1536px),calc(1080px / 1536px))'>x</div>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert!(
        (native.Zoom() - 800.0 / 1536.0).abs() < 0.000_01,
        "{}",
        native.Zoom()
    );
}
#[test]
fn document_native_style_ua_author_inline_inheritance_and_media() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body id=parent style='font-size:20px;color:red;white-space:pre'><span id=child style='width:30px;position:absolute;font-family:serif, sans-serif'></span><div id=hidden><b id=inside></b></div></body></html>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS("#child{width:50px !important;padding-inline-start:3px;opacity:.4;background-color:blue;border-top-style:solid;border-top-width:2.6px} #hidden{display:none} @media(min-width:700px){#child{font-weight:700}}"));
    let ua = crate::ParseCSS("body{display:block;margin:8px} span{display:inline}");
    let child = node(&owner, "child");
    let inside = node(&owner, "inside");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[ua]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let s = owner.GetDocument().ResolvedStyleFor(child).unwrap();
    let native = unsafe { &*s.native_style.Get() };
    assert_eq!(s.style.native_style, s.native_style.Get());
    assert_eq!(native.Display(), EDisplay::kBlock);
    assert_eq!(native.GetPosition(), EPosition::kAbsolute);
    assert_eq!(native.Width().Pixels(), 50.0);
    assert_eq!(native.PaddingLeft().Pixels(), 3.0);
    assert_eq!(native.Opacity(), 0.4);
    assert_eq!(native.BorderTopWidth(), 2);
    assert_eq!(s.style.paint.background_color.blue, 1.0);
    assert_eq!(s.style.paint.color.red, 1.0);
    assert_eq!(s.style.paint.opacity, 0.4);
    assert_eq!(native.GetFontDescription().ComputedSize(), 20.0);
    assert_eq!(native.GetFontDescription().Weight().ToFloat(), 700.0);
    let ext = s.style.extended.as_ref().unwrap();
    assert_eq!(ext.font_size, 20.0);
    assert_eq!(
        ext.white_space,
        layoutng::internal::layout_input::WhiteSpace::kPre
    );
    assert_eq!(ext.font_families, vec!["serif", "sans-serif"]);
    assert!(
        !owner
            .GetDocument()
            .ResolvedStyleFor(inside)
            .unwrap()
            .generates_box
    );
}

#[test]
fn css_module_visually_hidden_rule_retains_all_supported_declarations() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<h2 id=target class='MarketingHeader-module__visuallyHidden__sqKsl'>Navigation Menu</h2>",
    );
    let sheet = crate::ParseCSS(
        ".MarketingHeader-module__visuallyHidden__sqKsl{clip-path:inset(50%);white-space:nowrap;border:0;width:1px;height:1px;margin:0;padding:0;position:absolute;overflow:hidden}",
    );
    assert_eq!(sheet.rules.len(), 1);
    assert_eq!(
        sheet.rules[0].selector_text,
        ".MarketingHeader-module__visuallyHidden__sqKsl"
    );
    assert!(sheet.rules[0]
        .declaration_text
        .contains("position:absolute"));
    let declarations = crate::ParseCSSDeclarationList(&sheet.rules[0].declaration_text);
    assert!(declarations
        .iter()
        .any(|declaration| declaration.property == "position"));
    owner.GetDocumentMut().AppendStyleSheet(sheet);
    let target = node(&owner, "target");
    assert!(
        crate::persistent_selector::PersistentSelectorService::default()
            .Matches(
                owner.GetDocument(),
                target,
                ".MarketingHeader-module__visuallyHidden__sqKsl",
            )
            .unwrap()
    );
    let compiled = crate::css_selector_list::CSSSelectorList::AdoptSelectorVector(
        crate::parser::css_selector_parser::CSSSelectorParser::ParseSelectorWithOptions(
            &foundation::String::from(".MarketingHeader-module__visuallyHidden__sqKsl"),
            &crate::parser::css_selector_parser::SelectorParserContext {
                html: true,
                quirks: false,
            },
            crate::parser::css_nesting_type::CSSNestingType::kNone,
            &Default::default(),
        ),
    );
    assert_eq!(
        compiled
            .First()
            .unwrap()
            .SimpleSelectors()
            .find(|selector| selector.Match() == crate::css_selector::MatchType::kClass)
            .unwrap()
            .Value()
            .Utf8(),
        "MarketingHeader-module__visuallyHidden__sqKsl"
    );
    assert!(
        crate::persistent_selector::PersistentSelectorService::default()
            .MatchingCompiledSpecificity(owner.GetDocument(), target, &compiled)
            .is_some()
    );
    for complex in compiled.ComplexSelectors() {
        let mut simple: Vec<_> = complex.SimpleSelectors().cloned().collect();
        for selector in &mut simple {
            selector.SetLastInComplexSelector(false);
            selector.SetLastInSelectorList(false);
        }
        simple.last_mut().unwrap().SetLastInComplexSelector(true);
        let retained = crate::css_selector_list::CSSSelectorList::AdoptSelectorVector(simple);
        assert!(
            crate::persistent_selector::PersistentSelectorService::default()
                .MatchingCompiledSpecificity(owner.GetDocument(), target, &retained)
                .is_some()
        );
    }
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*resolved.native_style.Get() };
    assert_eq!(native.GetPosition(), EPosition::kAbsolute);
    assert_eq!(native.Width().Pixels(), 1.0);
    assert_eq!(native.Height().Pixels(), 1.0);
    assert_eq!(native.OverflowX(), EOverflow::kHidden);
    assert_eq!(native.OverflowY(), EOverflow::kHidden);
    assert!(native.HasClipPath());
    let projected = resolved.style.extended.as_ref().unwrap();
    assert_eq!(
        projected.overflow_x,
        layoutng::internal::layout_input::Overflow::kHidden
    );
    assert_eq!(
        projected.overflow_y,
        layoutng::internal::layout_input::Overflow::kHidden
    );
}

#[test]
fn native_list_style_none_and_position_cross_the_layout_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<ul id=none style='list-style:none'><li>x</li></ul><ol id=inside style='list-style:decimal inside'><li>y</li></ol>",
    );
    let none = node(&owner, "none");
    let inside = node(&owner, "inside");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let none = owner.GetDocument().ResolvedStyleFor(none).unwrap();
    let none = none.style.extended.as_ref().unwrap();
    assert_eq!(
        none.list_style_type,
        layoutng::internal::layout_input::ListStyleType::kNone
    );
    let inside = owner.GetDocument().ResolvedStyleFor(inside).unwrap();
    let inside = inside.style.extended.as_ref().unwrap();
    assert_eq!(
        inside.list_style_type,
        layoutng::internal::layout_input::ListStyleType::kDecimal
    );
    assert_eq!(
        inside.list_style_position,
        layoutng::internal::layout_input::ListStylePosition::kInside
    );
}
#[test]
fn unsupported_declarations_are_diagnostic_and_do_not_drop_valid_styles() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=target style='width:7px;grid-template-columns:repeat(2,1fr);padding-left:9px'></div>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(!engine.Diagnostics().is_empty());
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Width().Pixels(), 7.0);
    assert_eq!(native.PaddingLeft().Pixels(), 9.0);
    assert!(!owner.GetDocument().StyleState().all_dirty);
    let mut other = html::html_parser::ParseHTML("<div></div>");
    assert!(matches!(
        engine.Update(&mut other, &media(), &[]),
        Err(crate::DocumentStyleError::WrongDocument)
    ));
}

#[test]
fn svg_presentation_attributes_enter_the_author_cascade() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        r##"<svg xmlns="http://www.w3.org/2000/svg"><g id="group" transform="translate(-111.5 -66.5) scale(.35)"><path id="shape" fill="#15919d" d="M0 0L10 0L10 10Z"/></g></svg>"##,
    );
    let group = node(&owner, "group");
    let shape = node(&owner, "shape");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );

    let group_resolved = owner.GetDocument().ResolvedStyleFor(group).unwrap();
    let group = unsafe { &*group_resolved.native_style.Get() };
    assert_eq!(group.Transform().size(), 1);
    assert_eq!(
        group_resolved
            .style
            .paint
            .transform
            .as_ref()
            .unwrap()
            .operations[0]
            .matrix
            .values[12],
        -111.5
    );

    let shape_resolved = owner.GetDocument().ResolvedStyleFor(shape).unwrap();
    let shape = unsafe { &*shape_resolved.native_style.Get() };
    assert_eq!(
        shape.FillPaint().GetColor().GetColor().Param0(),
        0x15 as f32
    );
    assert_eq!(
        shape_resolved.style.paint.svg_fill.unwrap().red,
        0x15 as f32 / 255.0
    );
}

#[test]
fn unnamed_white_space_combination_crosses_the_layout_boundary() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{white-space:preserve-breaks nowrap}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert_eq!(
        native.GetWhiteSpaceCollapse(),
        layoutng_style::css::white_space::WhiteSpaceCollapse::kPreserveBreaks
    );
    assert_eq!(native.GetTextWrapMode(), foundation::TextWrapMode::kNowrap);
    assert_eq!(
        style.style.extended.as_ref().unwrap().white_space,
        layoutng::internal::layout_input::WhiteSpace::kPreserveBreaksNowrap
    );
}

#[test]
fn document_cascade_resolves_environment_values_and_records_style_state() {
    use crate::{
        css_value::CSSValuePayload,
        parser::{css_parser_mode::CSSParserMode, production_property_parser::ParseCustomProperty},
        resolver::production_style_builder::custom_properties::EnvironmentVariableResolver,
    };
    use foundation::{AtomicString, String};
    use std::rc::Rc;

    struct Environment(Rc<crate::production_css_value::CSSVariableData>);
    impl EnvironmentVariableResolver for Environment {
        fn ViewportSegmentsEnabled(&self) -> bool {
            false
        }
        fn ResolveEnvironmentVariable(
            &self,
            name: &AtomicString,
            indices: &[u32],
        ) -> Option<Rc<crate::production_css_value::CSSVariableData>> {
            (name == &AtomicString::from_str("safe-area-inset-top") && indices.is_empty())
                .then(|| self.0.clone())
        }
    }

    let value = ParseCustomProperty(
        "--environment-fixture",
        &String::from("23px"),
        false,
        CSSParserMode::kHTMLStandardMode,
    )
    .unwrap();
    let CSSValuePayload::kUnparsedDeclarationClass(value) = value.Value().Payload() else {
        panic!("environment fixture must retain its token stream")
    };
    let environment = Environment(value.data.clone());
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{padding-top:env(safe-area-inset-top,7px);padding-bottom:env(safe-area-inset-bottom,9px)}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine
        .UpdateWithEnvironment(&mut owner, &media(), &[], Some(&environment))
        .unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert_eq!(native.PaddingTop().Pixels(), 23.0);
    assert_eq!(native.PaddingBottom().Pixels(), 9.0);
    assert!(native.HasEnv());
    assert!(native.HasEnvSafeAreaInsetBottom());
}

#[test]
fn production_declarations_preserve_comment_token_boundaries() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{color:r/**/ed;width:1/**/0px;height:12px}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert!(!native.Width().IsFixed());
    assert_eq!(native.Height().Pixels(), 12.0);
    assert!(native.GetCurrentColor(None) == Color::kBlack);
}

#[test]
fn connected_style_element_text_is_collected_by_the_document_engine() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><html><head><style media='(min-width:700px)'>#target{width:23px;background-color:red}</style></head><body><div id=target></div></body></html>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(unsafe { &*style.native_style.Get() }.Width().Pixels(), 23.0);
    assert_eq!(style.style.paint.background_color.red, 1.0);
    assert_eq!(owner.GetDocument().ActiveStyleSheets().count(), 1);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(owner.GetDocument().ActiveStyleSheets().count(), 1);

    let mut narrow = media();
    narrow.viewport_width = 640.0;
    narrow.large_viewport_width = 640.0;
    engine.Update(&mut owner, &narrow, &[]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert!(!native.Width().IsFixed());
    assert_eq!(style.style.paint.background_color.alpha, 0.0);
    assert!(
        engine.TranslatedRuleShadowStats().skipped_owner_media >= 1,
        "{:?}",
        engine.TranslatedRuleShadowStats()
    );
}

#[test]
fn compound_class_background_and_zoom_cross_the_layout_boundary_once() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<button id=target class='button primary'>Text</button>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ":root{--design-vw:15.36px}.button{width:200px}.button.primary{zoom:.5;background:#04203b;font-size:40px;gap:calc(2.7 * var(--design-vw));border-radius:20px 50%;box-shadow:2px 4px 6px 8px}",
    ));
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
    assert_eq!(native.Width().Pixels(), 100.0);
    assert_eq!(native.GetFontDescription().ComputedSize(), 20.0);
    assert!((native.RowGap().as_ref().unwrap().Pixels() - 20.736).abs() < 0.001);
    assert!((native.ColumnGap().as_ref().unwrap().Pixels() - 20.736).abs() < 0.001);
    assert!(native.BackgroundColor().GetColor().Param2() > 0.1);
    assert!(native.HasAuthorBackground());
    assert_eq!(style.style.width, Some(200.0));
    assert_eq!(style.style.extended.as_ref().unwrap().font_size, 40.0);
    assert!(style.style.extended.as_ref().unwrap().has_author_background);
    let neutral_radii = style.style.paint.border_radii.unwrap();
    assert_eq!(neutral_radii.top_left.x, 20.0);
    assert_eq!(style.style.paint.border_radii_percentages.top_right.x, 50.0);
    assert_eq!(style.style.paint.box_shadows[0].offset.x, 2.0);
    assert_eq!(style.style.paint.box_shadows[0].blur_radius, 6.0);
    assert_eq!(style.style.paint.box_shadows[0].spread, 8.0);
    assert!((style.style.extended.as_ref().unwrap().column_gap.unwrap() - 41.472).abs() < 0.001);
    let used = layoutng::internal::css_zoom::ZoomedStyle(&style.style);
    assert_eq!(used.width, Some(100.0));
    assert_eq!(used.extended.as_ref().unwrap().font_size, 20.0);
    assert_eq!(used.paint.border_radii.unwrap().top_left.x, 10.0);
    assert_eq!(used.paint.border_radii_percentages.top_right.x, 50.0);
    assert_eq!(used.paint.box_shadows[0].offset.x, 1.0);
    assert_eq!(used.paint.box_shadows[0].blur_radius, 3.0);
}
#[test]
fn source_layer_priorities_revert_layer_and_live_dom_mutations() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target class=x></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS("@layer first,second;@layer first{.x{width:11px !important;height:13px}}@layer second{#target{width:22px !important;height:24px}}#target{width:33px !important;height:revert-layer}"));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let s = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(s.Width().Pixels(), 11.0);
    assert_eq!(s.Height().Pixels(), 24.0);
    owner.GetDocumentMut().SetAttribute(
        target,
        dom::persistent_document::DOMAttribute {
            prefix: String::new(),
            local_name: "style".into(),
            namespace_uri: String::new(),
            value: "width:44px!important".into(),
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let s = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(s.Width().Pixels(), 44.0);
    owner
        .GetDocumentMut()
        .RemoveAttributeDefault(target, "style");
    owner.GetDocumentMut().SetAttribute(
        target,
        dom::persistent_document::DOMAttribute {
            prefix: String::new(),
            local_name: "class".into(),
            namespace_uri: String::new(),
            value: "different".into(),
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let s = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(s.Width().Pixels(), 22.0);
}
#[test]
fn dynamic_style_element_replaces_owner_sheet_and_recalculates_connected_nodes() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style id=sheet>#target{width:10px}</style><div id=target></div>",
    );
    let target = node(&owner, "target");
    let sheet = node(&owner, "sheet");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    owner
        .GetDocumentMut()
        .SetTextContent(sheet, "#target{width:19px}".into());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(owner.GetDocument().ActiveStyleSheets().count(), 1);
    let s = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(s.Width().Pixels(), 19.0);
}

fn set_attribute(owner: &mut dom::DOM, index: usize, name: &str, value: &str) {
    owner.GetDocumentMut().SetAttribute(
        index,
        dom::persistent_document::DOMAttribute {
            local_name: name.into(),
            value: value.into(),
            ..Default::default()
        },
    );
}
fn native_width(owner: &dom::DOM, index: usize) -> f32 {
    unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(index)
            .unwrap()
            .native_style
            .Get()
    }
    .Width()
    .Pixels()
}
fn native_pointer(
    owner: &dom::DOM,
    index: usize,
) -> *mut layoutng_style::style::computed_style::ComputedStyle {
    owner
        .GetDocument()
        .ResolvedStyleFor(index)
        .unwrap()
        .native_style
        .Get()
}
#[test]
fn incremental_inline_and_class_changes_retain_unrelated_native_styles() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><div id=target class=small><b id=descendant></b></div><section id=unrelated><b></b><b></b><b></b></section></body></html>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS(".small{width:10px}.large{width:20px}"));
    let target = node(&owner, "target");
    let descendant = node(&owner, "descendant");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let full = owner.GetDocument().StyleState().stats.resolved_nodes;
    let unrelated_style = owner.GetDocument().ResolvedStyleHandle(unrelated).unwrap();
    let child_pointer = native_pointer(&owner, descendant);
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().IsEmpty());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
    assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 0);
    set_attribute(&mut owner, target, "class", "large");
    let impact = engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(impact.layout && !impact.reattach);
    assert_eq!(native_width(&owner, target), 20.0);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);
    assert!(owner.GetDocument().StyleState().stats.resolved_nodes < full);
    assert_eq!(native_pointer(&owner, descendant), child_pointer);
    assert!(std::sync::Arc::ptr_eq(
        &unrelated_style,
        &owner.GetDocument().ResolvedStyleHandle(unrelated).unwrap()
    ));
    set_attribute(&mut owner, target, "style", "opacity:.25");
    let impact = engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(impact.paint && !impact.layout && !impact.reattach);
    assert_eq!(owner.GetDocument().StyleState().stats.changed_nodes, 1);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);
    // A different selector state with an equal cascade preserves native identity.
    let pointer = native_pointer(&owner, target);
    set_attribute(&mut owner, target, "class", "large extra");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().IsEmpty());
    assert_eq!(owner.GetDocument().StyleState().stats.changed_nodes, 0);
    assert_eq!(native_pointer(&owner, target), pointer);
}
#[test]
fn incremental_inheritance_and_combinator_invalidation_resolve_only_changed_targets() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><section id=host><b id=target class=item></b><b id=plain></b></section><section id=unrelated><b></b><b></b></section></body></html>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS(".active > .item{width:31px}"));
    let host = node(&owner, "host");
    let target = node(&owner, "target");
    let plain = node(&owner, "plain");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let full = owner.GetDocument().StyleState().stats.resolved_nodes;
    let plain_pointer = native_pointer(&owner, plain);
    let unrelated_pointer = native_pointer(&owner, unrelated);
    set_attribute(&mut owner, host, "class", "active");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, target), 31.0);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 2);
    assert_eq!(native_pointer(&owner, plain), plain_pointer);
    set_attribute(&mut owner, host, "style", "color:red");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 3);
    assert!(owner.GetDocument().StyleState().stats.resolved_nodes < full);
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(plain)
            .unwrap()
            .style
            .paint
            .color
            .red,
        1.0
    );
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
    owner.GetDocumentMut().RemoveAttributeDefault(host, "class");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(unsafe { &*native_pointer(&owner, target) }.Width().IsAuto());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 2);
}
#[test]
fn incremental_sheet_replacement_and_removal_invalidate_old_and_new_targets() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><head><style id=sheet>#first{width:10px}</style></head><body><div id=first></div><div id=second></div><div id=unrelated><b></b><b></b></div></body></html>",
    );
    let sheet = node(&owner, "sheet");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let full = owner.GetDocument().StyleState().stats.resolved_nodes;
    let unrelated_pointer = native_pointer(&owner, unrelated);
    owner
        .GetDocumentMut()
        .SetTextContent(sheet, "#second{width:22px}".into());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(unsafe { &*native_pointer(&owner, first) }.Width().IsAuto());
    assert_eq!(native_width(&owner, second), 22.0);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 2);
    assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 1);
    assert!(owner.GetDocument().StyleState().stats.resolved_nodes < full);
    owner.GetDocumentMut().Remove(sheet);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(unsafe { &*native_pointer(&owner, second) }.Width().IsAuto());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
}
#[test]
fn incremental_append_and_reverse_position_invalidation_preserve_other_branches() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><ul id=list><li id=first></li><li id=last></li></ul><section id=unrelated><b></b><b></b><b></b></section></body></html>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "li:last-child{width:12px}li:nth-last-child(2){height:15px}",
    ));
    let list = node(&owner, "list");
    let first = node(&owner, "first");
    let last = node(&owner, "last");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let full = owner.GetDocument().StyleState().stats.resolved_nodes;
    let unrelated_pointer = native_pointer(&owner, unrelated);
    let inserted = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "li".into());
    owner.GetDocumentMut().AppendChild(list, inserted);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, inserted), 12.0);
    assert!(unsafe { &*native_pointer(&owner, last) }.Width().IsAuto());
    assert_eq!(
        unsafe { &*native_pointer(&owner, last) }.Height().Pixels(),
        15.0
    );
    assert!(unsafe { &*native_pointer(&owner, first) }.Height().IsAuto());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 3);
    assert!(owner.GetDocument().StyleState().stats.resolved_nodes < full);
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
}

#[test]
fn incremental_explicit_inheritance_and_display_suppression_follow_parent_changes() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><div id=parent style='opacity:.1;display:none'><span id=child style='opacity:inherit'><b id=grandchild style='opacity:inherit'></b></span></div><section id=unrelated><b></b><b></b></section></body></html>",
    );
    let parent = node(&owner, "parent");
    let child = node(&owner, "child");
    let grandchild = node(&owner, "grandchild");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let unrelated_pointer = native_pointer(&owner, unrelated);
    assert!(
        !owner
            .GetDocument()
            .ResolvedStyleFor(child)
            .unwrap()
            .generates_box
    );
    set_attribute(&mut owner, parent, "style", "opacity:.2;display:contents");
    let impact = engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(impact.reattach);
    assert!(
        owner
            .GetDocument()
            .ResolvedStyleFor(child)
            .unwrap()
            .generates_box
    );
    assert!(
        owner
            .GetDocument()
            .ResolvedStyleFor(grandchild)
            .unwrap()
            .generates_box
    );
    assert_eq!(unsafe { &*native_pointer(&owner, child) }.Opacity(), 0.2);
    assert_eq!(
        unsafe { &*native_pointer(&owner, grandchild) }.Opacity(),
        0.2
    );
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 3);
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
    set_attribute(&mut owner, parent, "style", "opacity:.3;display:contents");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(
        unsafe { &*native_pointer(&owner, grandchild) }.Opacity(),
        0.3
    );
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 3);
}
#[test]
fn incremental_child_state_and_sibling_selectors_invalidate_beyond_the_container() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><div id=container></div><div id=target></div><section id=unrelated><b></b><b></b></section></body></html>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("#container:empty + #target{width:14px}"));
    let container = node(&owner, "container");
    let target = node(&owner, "target");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, target), 14.0);
    let unrelated_pointer = native_pointer(&owner, unrelated);
    let inserted = owner
        .GetDocumentMut()
        .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "b".into());
    owner.GetDocumentMut().AppendChild(container, inserted);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(unsafe { &*native_pointer(&owner, target) }.Width().IsAuto());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 2);
    owner.GetDocumentMut().Remove(inserted);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, target), 14.0);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
}
#[test]
fn incremental_has_and_filtered_nth_selectors_track_mutated_selector_dependencies() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><section id=host><b id=first class=selected></b><b id=second class=selected></b></section><section id=unrelated><b></b><b></b></section></body></html>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#host:has(.active){width:25px}.selected:nth-child(2 of .selected){height:17px}",
    ));
    let host = node(&owner, "host");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(
        unsafe { &*native_pointer(&owner, second) }
            .Height()
            .Pixels(),
        17.0
    );
    let unrelated_pointer = native_pointer(&owner, unrelated);
    set_attribute(&mut owner, first, "class", "active");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, host), 25.0);
    assert!(unsafe { &*native_pointer(&owner, second) }
        .Height()
        .IsAuto());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 3);
    assert_eq!(native_pointer(&owner, unrelated), unrelated_pointer);
}
#[test]
fn incremental_appended_sheet_resolves_its_targets_without_revisiting_older_rules() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<html><body><div id=first></div><div id=second></div><section id=unrelated><b></b><b></b></section></body></html>",
    );
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("#first{width:10px}"));
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let first_pointer = native_pointer(&owner, first);
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("#second{width:20px}"));
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(native_width(&owner, second), 20.0);
    assert_eq!(native_pointer(&owner, first), first_pointer);
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 1);
    assert_eq!(owner.GetDocument().StyleState().stats.rule_sets_built, 1);
    // Subsequent selector invalidation compares signatures for the new order.
    set_attribute(&mut owner, first, "class", "unrelated");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().IsEmpty());
    assert_eq!(native_pointer(&owner, first), first_pointer);
}

#[test]
fn production_pseudo_styles_have_independent_cascade_and_native_content() {
    use layoutng_style::style::{
        computed_style_constants::PseudoId, content_data::TextContentData,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>#target{display:block;color:red;font-size:20px} #target::before{content:'wrong';color:blue} .host::before{content:attr(data-label) ' \\41';color:green} #target::after{content:'';display:contents} #target::first-letter{font-size:40px} input::placeholder{color:blue;font-style:italic}</style><div id=target class=host data-label=hello style='color:red;content:none'>Alpha</div><input id=control placeholder=hint>",
    );
    // A normal selector in the same list applies only to its ordinary subject.
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#unmatched#another, .host::before{content:attr(data-label) ' \\41';color:blue !important}",
    ));
    let target = node(&owner, "target");
    let control = node(&owner, "control");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    // #target has greater specificity than .host for content; inline content
    // belongs to the originating element and does not enter the pseudo cascade.
    let before = style.before.as_ref().unwrap();
    assert_eq!(before.text, "wrong");
    assert_eq!(before.style.paint.color.blue, 1.0);
    assert_eq!(style.style.paint.color.red, 1.0);
    let native = unsafe { &*before.native_style.Get() };
    let origin = unsafe { &*style.native_style.Get() };
    assert!(origin.HasPseudoElementStyle(PseudoId::kPseudoIdBefore));
    assert!(origin.CanGeneratePseudoElement(PseudoId::kPseudoIdBefore));
    assert_eq!(
        origin.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore),
        before.native_style.Get()
    );
    assert_eq!(native.StyleType(), PseudoId::kPseudoIdBefore);
    assert_eq!(native.GetFontDescription().ComputedSize(), 20.0);
    assert_eq!(before.style.native_style, before.native_style.Get());
    let data = native.GetContentData().unwrap();
    assert!(unsafe { (&*data).IsText() });
    assert_eq!(
        unsafe { (&*(data as *const TextContentData)).GetText().Utf8() },
        "wrong"
    );
    let after = style.after.as_ref().unwrap();
    assert_eq!(after.text, "");
    assert!(after.display_contents);
    assert_eq!(
        unsafe { &*style.first_letter.as_ref().unwrap().native_style.Get() }
            .GetFontDescription()
            .ComputedSize(),
        40.0
    );
    let placeholder = owner
        .GetDocument()
        .ResolvedStyleFor(control)
        .unwrap()
        .placeholder
        .as_ref()
        .unwrap();
    assert_eq!(
        unsafe { &*placeholder.native_style.Get() }.StyleType(),
        PseudoId::kPseudoIdPlaceholder
    );
    assert_eq!(placeholder.style.paint.color.blue, 1.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.skipped_pseudo, 0, "{stats:?}");
    assert_eq!(
        stats.translated_style_cascade_pseudo_fallbacks, 0,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_pseudo_attempts,
        stats.translated_style_cascade_pseudo_applied,
        "{stats:?}"
    );
}

#[test]
fn production_marker_pseudo_owns_cascade_cache_and_content() {
    use layoutng_style::{
        css::white_space::EWhiteSpace, style::computed_style_constants::PseudoId,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let marker_ua = crate::ParseUserAgentCSSWithRole(
        "::marker, ::before::marker, ::after::marker { unicode-bidi:isolate; font-variant-numeric:tabular-nums; text-transform:none; text-indent:0 !important; text-align:start !important; text-align-last:auto !important; }",
        cssom::CSSUserAgentStyleSheetRole::kMarker,
    );
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>#item{display:list-item}#item::marker{content:'owned';color:rgb(0 128 0);font-size:24px;width:123px}</style><ul><li id=item>text</li></ul><div id=ordinary>plain</div>",
    );
    let item = node(&owner, "item");
    let ordinary = node(&owner, "ordinary");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[marker_ua]).unwrap();

    let resolved = owner.GetDocument().ResolvedStyleFor(item).unwrap();
    let marker = resolved.marker.as_ref().expect("list item owns ::marker");
    let native = unsafe { &*marker.native_style.Get() };
    let originating = unsafe { &*resolved.native_style.Get() };
    assert_eq!(marker.text, "owned");
    assert_eq!(native.StyleType(), PseudoId::kPseudoIdMarker);
    assert_eq!(native.GetFontDescription().ComputedSize(), 24.0);
    assert_eq!(native.Display(), EDisplay::kInlineBlock);
    assert_eq!(native.WhiteSpace(), EWhiteSpace::kPre);
    // width is not valid on ::marker and must be filtered by RuleData's
    // ValidPropertyFilter rather than leaking through the document adapter.
    assert!(native.Width().IsAuto());
    assert_eq!(
        originating.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdMarker),
        marker.native_style.Get()
    );
    assert!(originating.HasPseudoElementStyle(PseudoId::kPseudoIdMarker));
    assert!(owner
        .GetDocument()
        .ResolvedStyleFor(ordinary)
        .unwrap()
        .marker
        .is_none());
}

#[test]
fn nested_generated_marker_owns_originating_pseudo_cascade_and_invalidation() {
    use layoutng_style::style::computed_style_constants::PseudoId;
    let _heap = foundation::LayoutHeapScope::new();
    let marker_ua = crate::ParseUserAgentCSSWithRole(
        "::marker, ::before::marker, ::after::marker { unicode-bidi:isolate; font-variant-numeric:tabular-nums; text-transform:none; text-indent:0 !important; text-align:start !important; text-align-last:auto !important; }",
        cssom::CSSUserAgentStyleSheetRole::kMarker,
    );
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><style>\
         #item::before{content:'body';display:list-item;color:rgb(255 0 0);--marker-size:23px}\
         #item::before::marker{content:'before-marker';color:rgb(0 0 255);font-size:var(--marker-size);width:123px}\
         #item.swap::before::marker{content:'updated-marker'}\
         #item::after{content:'tail';display:list-item}\
         #item::after::marker{content:'after-marker'}\
         </style><div id=item>origin</div>",
    );
    let item = node(&owner, "item");
    let mut engine = StyleEngine::new(&owner);
    engine
        .Update(&mut owner, &media(), &[marker_ua.clone()])
        .unwrap();

    let resolved = owner.GetDocument().ResolvedStyleFor(item).unwrap();
    assert!(
        resolved.marker.is_none(),
        "the DOM element is not a list item"
    );
    let before = resolved.before.as_ref().expect("generated ::before");
    let marker = before.marker.as_deref().expect("::before owns ::marker");
    let before_native = unsafe { &*before.native_style.Get() };
    let marker_native = unsafe { &*marker.native_style.Get() };
    assert_eq!(marker.text, "before-marker");
    assert_eq!(marker_native.StyleType(), PseudoId::kPseudoIdMarker);
    assert_eq!(marker_native.GetFontDescription().ComputedSize(), 23.0);
    assert!(marker_native.Width().IsAuto());
    assert!(before_native.HasPseudoElementStyle(PseudoId::kPseudoIdMarker));
    assert_eq!(
        before_native.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdMarker),
        marker.native_style.Get()
    );
    assert_eq!(
        resolved
            .after
            .as_ref()
            .and_then(|pseudo| pseudo.marker.as_deref())
            .map(|marker| marker.text.as_str()),
        Some("after-marker")
    );
    let old_marker = marker.native_style.Get();

    set_attribute(&mut owner, item, "class", "swap");
    engine.Update(&mut owner, &media(), &[marker_ua]).unwrap();
    let marker = owner
        .GetDocument()
        .ResolvedStyleFor(item)
        .unwrap()
        .before
        .as_ref()
        .unwrap()
        .marker
        .as_deref()
        .unwrap();
    assert_eq!(marker.text, "updated-marker");
    assert_ne!(marker.native_style.Get(), old_marker);
}

#[test]
fn translated_style_cascade_publishes_pseudo_native_style_with_exact_stats() {
    use layoutng_style::style::{
        computed_style_constants::PseudoId, content_data::TextContentData,
    };

    let _heap = foundation::LayoutHeapScope::new();
    let mut owner =
        html::html_parser::ParseHTML("<!doctype html><div id=target class=host>originating</div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".host{display:block;font-size:19px}.host::before{content:'translated';display:block;color:rgb(0 0 255)}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
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
    assert_eq!(
        stats.translated_style_cascade_pseudo_attempts,
        stats.translated_style_cascade_pseudo_applied
            + stats.translated_style_cascade_pseudo_fallbacks,
        "each pseudo trial must either publish or wholly fall back: {stats:?}"
    );

    let resolved = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let pseudo = resolved.before.as_ref().expect("generated ::before");
    let native = unsafe { &*pseudo.native_style.Get() };
    assert_eq!(native.StyleType(), PseudoId::kPseudoIdBefore);
    assert_eq!(native.Display(), foundation::EDisplay::kBlock);
    assert_eq!(native.GetFontDescription().ComputedSize(), 19.0);
    assert_eq!(pseudo.style.paint.color.blue, 1.0);
    let content = native.GetContentData().expect("native pseudo content");
    assert!(unsafe { (&*content).IsText() });
    assert_eq!(
        unsafe { (&*(content as *const TextContentData)).GetText().Utf8() },
        "translated"
    );
}

#[test]
fn registered_property_owner_is_dependency_scoped_and_reports_exact_apply() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><div id=plain class=plain></div>\
         <div id=parent class=registered-parent><div id=dependent class=dependent></div></div>\
         <div id=own class=own></div><div id=invalid class=invalid></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "@property --size{syntax:'<length>';inherits:false;initial-value:17px}\
         .plain{display:block}\
         .registered-parent{--size:31px;display:block}\
         .dependent{display:block;width:var(--size)}\
         .own{--size:41px;display:block;width:var(--size)}\
         .invalid{--size:red;display:block;width:var(--size)}",
    ));
    let plain = node(&owner, "plain");
    let dependent = node(&owner, "dependent");
    let own = node(&owner, "own");
    let invalid = node(&owner, "invalid");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_cascade_attempts, 5, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_applied, 5, "{stats:?}");
    assert_eq!(stats.translated_style_cascade_fallbacks, 0, "{stats:?}");
    assert_eq!(
        stats.translated_style_cascade_registered_attempts, 4,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_registered_applied, 4,
        "{stats:?}"
    );
    assert_eq!(
        stats.translated_style_cascade_registered_fallbacks, 0,
        "{stats:?}"
    );
    let plain_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(plain)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(plain_style.Display(), foundation::EDisplay::kBlock);
    let dependent_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(dependent)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(dependent_style.Display(), foundation::EDisplay::kBlock);
    assert_eq!(dependent_style.Width().Pixels(), 17.0);
    let own_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(own)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(own_style.Display(), foundation::EDisplay::kBlock);
    assert_eq!(own_style.Width().Pixels(), 41.0);
    let invalid_style = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(invalid)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(invalid_style.Display(), foundation::EDisplay::kBlock);
    assert_eq!(invalid_style.Width().Pixels(), 17.0);
}

#[test]
fn production_pseudo_cache_retains_suppressed_attr_content_and_invalidates_it() {
    use layoutng_style::style::{
        computed_style_constants::PseudoId, content_data::TextContentData,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>.host::before{content:attr(data-label);display:none} .host.show::before{display:block} .host::after{content:none}</style><div id=target class=host data-label=one></div>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let old = owner.GetDocument().ResolvedStyleHandle(target).unwrap();
    assert!(old.before.is_none());
    assert!(old.after.is_none());
    let origin = unsafe { &*old.native_style.Get() };
    assert!(origin.HasPseudoElementStyle(PseudoId::kPseudoIdBefore));
    assert!(origin.HasPseudoElementStyle(PseudoId::kPseudoIdAfter));
    let cached = origin.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore);
    let before = unsafe { &*cached };
    assert!(before.HasAttrFunction());
    assert_eq!(before.Display(), foundation::EDisplay::kNone);
    assert!(unsafe {
        &*origin.GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdAfter)
    }
    .ContentPreventsBoxGeneration());
    set_attribute(&mut owner, target, "data-label", "two");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let current = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert!(current.before.is_none());
    let changed = unsafe { &*current.native_style.Get() }
        .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore);
    assert_ne!(changed, cached);
    let content = unsafe { &*changed }.GetContentData().unwrap();
    assert_eq!(
        unsafe { &*(content as *const TextContentData) }
            .GetText()
            .Utf8(),
        "two"
    );
    set_attribute(&mut owner, target, "class", "host show");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    let current = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    assert_eq!(current.before.as_ref().unwrap().text, "two");
    assert_eq!(
        unsafe { &*current.native_style.Get() }
            .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore),
        current.before.as_ref().unwrap().native_style.Get()
    );
    set_attribute(&mut owner, target, "class", "removed");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    let current = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let origin = unsafe { &*current.native_style.Get() };
    assert!(!origin.HasAnyPseudoElementStyles());
    assert!(origin
        .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore)
        .is_null());
}

#[test]
fn production_pseudo_generation_respects_contents_and_suppressed_ancestors() {
    use layoutng_style::style::computed_style_constants::PseudoId;
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>.host{display:contents}.host::before{content:'yes'}.host::first-letter{font-size:40px}</style><div id=contents class=host>Text</div><section style='display:none'><div id=suppressed class=host>Text</div></section>",
    );
    let contents = node(&owner, "contents");
    let suppressed = node(&owner, "suppressed");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let contents = owner.GetDocument().ResolvedStyleFor(contents).unwrap();
    assert!(contents.before.is_some());
    assert!(contents.first_letter.is_none());
    assert!(unsafe { &*contents.native_style.Get() }
        .HasPseudoElementStyle(PseudoId::kPseudoIdFirstLetter));
    assert!(!unsafe { &*contents.native_style.Get() }
        .CanGeneratePseudoElement(PseudoId::kPseudoIdFirstLetter));
    let suppressed = owner.GetDocument().ResolvedStyleFor(suppressed).unwrap();
    assert!(suppressed.before.is_none());
    assert!(suppressed.first_letter.is_none());
    assert!(unsafe { &*suppressed.native_style.Get() }
        .GetCachedPseudoElementStyleWithoutArgument(PseudoId::kPseudoIdBefore)
        .is_null());
}

#[test]
fn production_pseudo_local_invalidation_and_unchanged_frame_reuse() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>.host{display:block;color:red} .host::before{content:attr(data-label) ' \\41'} .hidden::before{display:none} .host::after{content:none} .ancestor.on .host::after{content:'tail'}</style><section id=ancestor class=ancestor><div id=target class=host data-label=one>Text</div></section><div id=unrelated class=host data-label=other>Other</div>",
    );
    let target = node(&owner, "target");
    let ancestor = node(&owner, "ancestor");
    let unrelated = node(&owner, "unrelated");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let old = owner.GetDocument().ResolvedStyleHandle(target).unwrap();
    let other = owner.GetDocument().ResolvedStyleHandle(unrelated).unwrap();
    assert_eq!(old.before.as_ref().unwrap().text, "one A");
    assert!(old.after.is_none());
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().IsEmpty());
    assert_eq!(owner.GetDocument().StyleState().stats.resolved_nodes, 0);
    assert!(std::sync::Arc::ptr_eq(
        &old,
        &owner.GetDocument().ResolvedStyleHandle(target).unwrap()
    ));
    set_attribute(&mut owner, target, "data-label", "two");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .before
            .as_ref()
            .unwrap()
            .text,
        "two A"
    );
    assert!(std::sync::Arc::ptr_eq(
        &other,
        &owner.GetDocument().ResolvedStyleHandle(unrelated).unwrap()
    ));
    set_attribute(&mut owner, ancestor, "class", "ancestor on");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .after
            .as_ref()
            .unwrap()
            .text,
        "tail"
    );
    let before = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .before
        .as_ref()
        .unwrap()
        .native_style
        .Get();
    set_attribute(&mut owner, target, "class", "host extra");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().IsEmpty());
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .before
            .as_ref()
            .unwrap()
            .native_style
            .Get(),
        before
    );
    set_attribute(&mut owner, target, "class", "host hidden");
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    assert!(owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .before
        .is_none());
}

#[test]
fn production_custom_properties_feed_cascade_and_invalidate_inheriting_children() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=parent style='--size:31px;--tone:blue'><span id=child style='display:block;width:var(--size);height:var(--missing,9px);color:var(--tone)'></span></div>",
    );
    let parent = node(&owner, "parent");
    let child = node(&owner, "child");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let resolved = owner.GetDocument().ResolvedStyleFor(child).unwrap();
    let style = unsafe { &*resolved.native_style.Get() };
    assert_eq!(style.Width().Pixels(), 31.0);
    assert_eq!(style.Height().Pixels(), 9.0);
    assert_eq!(resolved.style.paint.color.blue, 1.0);

    set_attribute(&mut owner, parent, "style", "--size:47px;--tone:red");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let resolved = owner.GetDocument().ResolvedStyleFor(child).unwrap();
    let style = unsafe { &*resolved.native_style.Get() };
    assert_eq!(style.Width().Pixels(), 47.0);
    assert_eq!(resolved.style.paint.color.red, 1.0);
}

#[test]
fn production_inherited_nested_custom_property_resolves_border_color() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style>:root{--base:#d1d9e0;--alias:var(--base)}#target{border:1px solid transparent;border-color:var(--alias)}</style><button id=target>Text</button>",
    );
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let color = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .style
        .paint
        .border_colors[0];
    assert!((color.red - 209.0 / 255.0).abs() < 0.001, "{color:?}");
    assert!((color.green - 217.0 / 255.0).abs() < 0.001, "{color:?}");
    assert!((color.blue - 224.0 / 255.0).abs() < 0.001, "{color:?}");
    assert!((color.alpha - 1.0).abs() < 0.001, "{color:?}");
}

#[test]
fn production_pseudo_sheet_removal_and_unsupported_requests_do_not_fake_content() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<style id=sheet>#target::before{content:'ok'} #target::after{content:'fallback';content:counter(item, symbols('x'))} #target::marker{color:red}</style><div id=target></div>",
    );
    let target = node(&owner, "target");
    let sheet = node(&owner, "sheet");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(engine
        .Diagnostics()
        .iter()
        .any(|d| matches!(d, crate::DocumentStyleError::Unsupported(_))));
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .before
            .as_ref()
            .unwrap()
            .text,
        "ok"
    );
    // An explicitly unsupported consumer is diagnosed and omitted through
    // the ordinary production parser; the valid earlier declaration remains.
    assert_eq!(
        owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .after
            .as_ref()
            .unwrap()
            .text,
        "fallback"
    );
    assert!(engine.Diagnostics().iter().any(|d| matches!(d,
        crate::DocumentStyleError::Property { property: foundation::CSSPropertyID::kContent, operation, .. }
            if operation.contains("ConsumeCounterStyleSymbolsFunction"))));
    owner
        .GetDocumentMut()
        .SetTextContent(sheet, "#target::before{content:normal}".into());
    assert!(engine.Update(&mut owner, &media(), &[]).unwrap().reattach);
    assert!(owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .before
        .is_none());
}

#[test]
fn production_animation_transition_cascade_writes_native_timing_lists() {
    use layoutng_style::style::css_timing_data::{
        PlaybackDirection, TimingFunction, TransitionBehavior, TransitionProperty,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=parent style='animation-delay:250ms'><div id=target style='animation-delay:inherit;animation-duration:2s;transition-property:opacity,--progress;transition-behavior:allow-discrete'></div></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{animation-duration:150ms!important;animation-direction:alternate-reverse;animation-timing-function:steps(3,jump-none);transition-duration:1s,200ms;transition-delay:-300ms}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let initial = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .native_style
        .clone();
    let native = unsafe { &*initial.Get() };
    let animation = unsafe { &*native.Animations().Get() };
    assert_eq!(animation.DurationList(), &[Some(0.15)]);
    assert_eq!(animation.DelayStartList()[0].time_delay, 0.25);
    assert_eq!(
        animation.DirectionList(),
        &[PlaybackDirection::ALTERNATE_REVERSE]
    );
    assert!(matches!(
        &**animation.TimingFunctionList().first().unwrap(),
        TimingFunction::Steps {
            number_of_steps: 3,
            ..
        }
    ));
    let transition = unsafe { &*native.Transitions().Get() };
    assert_eq!(transition.DurationList(), &[Some(1.0), Some(0.2)]);
    assert_eq!(transition.DelayStartList()[0].time_delay, -0.3);
    assert_eq!(
        transition.BehaviorList(),
        &[TransitionBehavior::kAllowDiscrete]
    );
    assert_eq!(
        transition.PropertyList()[0],
        TransitionProperty::Known(foundation::CSSPropertyID::kOpacity)
    );
    assert!(
        matches!(&transition.PropertyList()[1], TransitionProperty::Unknown(name) if name.Utf8() == "--progress")
    );

    set_attribute(
        &mut owner,
        target,
        "style",
        "animation-delay:500ms;transition-property:none;transition-duration:3s!important",
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let updated = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let updated = unsafe { &*updated.native_style.Get() };
    assert_ne!(updated.Animations().Get(), native.Animations().Get());
    assert_ne!(updated.Transitions().Get(), native.Transitions().Get());
    assert_eq!(
        unsafe { &*updated.Animations().Get() }.DelayStartList()[0].time_delay,
        0.5
    );
    assert_eq!(
        unsafe { &*updated.Transitions().Get() }.DurationList(),
        &[Some(3.0)]
    );
    assert_eq!(
        unsafe { &*updated.Transitions().Get() }.PropertyList(),
        &[TransitionProperty::None]
    );
    assert_eq!(animation.DelayStartList()[0].time_delay, 0.25);
    assert_eq!(transition.DurationList(), &[Some(1.0), Some(0.2)]);
}

#[test]
fn production_position_and_repeat_values_reach_native_style() {
    use layoutng_style::style::computed_style_constants::{BackgroundEdgeOrigin, EFillRepeat};
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{object-position:left 10px top 20px;perspective-origin:25% 75%;transform-origin:10% 20% -3px;background-position:right 7px bottom 9px;background-repeat:round space;mask-position:left 3px top 4px;mask-repeat:no-repeat round}",
    ));
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
    assert_eq!(native.ObjectPosition().X().Pixels(), 10.0);
    assert_eq!(native.ObjectPosition().Y().Pixels(), 20.0);
    assert_eq!(native.PerspectiveOrigin().X().PercentValue(), 25.0);
    assert_eq!(native.PerspectiveOrigin().Y().PercentValue(), 75.0);
    assert_eq!(native.GetTransformOrigin().X().PercentValue(), 10.0);
    assert_eq!(native.GetTransformOrigin().Y().PercentValue(), 20.0);
    assert_eq!(native.GetTransformOrigin().Z(), -3.0);
    let background = native.BackgroundLayers();
    assert_eq!(background.PositionX().Pixels(), 7.0);
    assert_eq!(background.PositionY().Pixels(), 9.0);
    assert_eq!(background.BackgroundXOrigin(), BackgroundEdgeOrigin::kRight);
    assert_eq!(
        background.BackgroundYOrigin(),
        BackgroundEdgeOrigin::kBottom
    );
    assert_eq!(background.Repeat().x, EFillRepeat::kRoundFill);
    assert_eq!(background.Repeat().y, EFillRepeat::kSpaceFill);
    let mask = native.MaskLayers();
    assert_eq!(mask.PositionX().Pixels(), 3.0);
    assert_eq!(mask.PositionY().Pixels(), 4.0);
    assert_eq!(mask.Repeat().x, EFillRepeat::kNoRepeatFill);
    assert_eq!(mask.Repeat().y, EFillRepeat::kRoundFill);
}

#[test]
fn production_scope_root_limit_nested_and_self_limit() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<main class=outer id=root><p id=direct class=item></p><section class=inner><p id=nested class=item></p></section><aside class=stop><p id=limited class=item></p><section class=inner><p id=blocked class=item></p></section></aside></main><p id=outside class=item></p>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(".item{width:1px;height:2px} @scope (.outer) to (.stop){.item{width:10px} :scope{padding-left:3px} @scope (.inner){& > .item{height:20px}}} @scope (.outer) to (:scope){.item{width:99px}}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    for (id, width, height) in [
        ("direct", 10.0, 2.0),
        ("nested", 10.0, 20.0),
        ("limited", 1.0, 2.0),
        ("blocked", 1.0, 2.0),
        ("outside", 1.0, 2.0),
    ] {
        let style = owner
            .GetDocument()
            .ResolvedStyleFor(node(&owner, id))
            .unwrap();
        let native = unsafe { &*style.native_style.Get() };
        assert_eq!(
            (native.Width().Pixels(), native.Height().Pixels()),
            (width, height),
            "{id}: {:?}",
            engine.Diagnostics()
        );
    }
    let root = owner
        .GetDocument()
        .ResolvedStyleFor(node(&owner, "root"))
        .unwrap();
    assert_eq!(
        unsafe { &*root.native_style.Get() }.PaddingLeft().Pixels(),
        3.0
    );
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}

#[test]
fn production_scope_proximity_precedes_order_after_specificity_and_layer() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<main class=far><section class=near><p id=target class=item></p></section></main>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS("@layer first,second; @scope (.near){.item{width:11px;height:12px}} @scope (.far){.item{width:21px} #target{height:22px}} .item{width:31px} @layer first{@scope (.near){.item{padding-left:13px}}} @layer second{@scope (.far){.item{padding-left:23px}}}"));
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node(&owner, "target"))
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(native.Width().Pixels(), 11.0);
    assert_eq!(native.Height().Pixels(), 22.0);
    assert_eq!(native.PaddingLeft().Pixels(), 23.0);
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}

#[test]
fn production_scope_implicit_owner_and_root_limit_mutation_invalidation() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><main id=root class=active><style>@scope{.item{width:7px}} @scope (.active) to (.stop){.item{height:9px}}</style><section id=limit><p id=target class=item></p></section></main><p id=outside class=item></p>",
    );
    // An ownerless implicit scope must never become a document-wide rule.
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("@scope{.item{width:99px}}"));
    let root = node(&owner, "root");
    let limit = node(&owner, "limit");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    let read = |owner: &dom::DOM| {
        let n = unsafe {
            &*owner
                .GetDocument()
                .ResolvedStyleFor(target)
                .unwrap()
                .native_style
                .Get()
        };
        (
            n.Width().Pixels(),
            n.Height().IsFixed().then(|| n.Height().Pixels()),
        )
    };
    assert_eq!(read(&owner), (7.0, Some(9.0)));
    let outside = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(node(&owner, "outside"))
            .unwrap()
            .native_style
            .Get()
    };
    assert!(!outside.Width().IsFixed());
    owner.GetDocumentMut().SetAttribute(
        limit,
        dom::persistent_document::DOMAttribute {
            local_name: "class".into(),
            value: "stop".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(read(&owner), (7.0, None));
    owner.GetDocumentMut().SetAttribute(
        limit,
        dom::persistent_document::DOMAttribute {
            local_name: "class".into(),
            value: "".into(),
            ..Default::default()
        },
    );
    owner.GetDocumentMut().SetAttribute(
        root,
        dom::persistent_document::DOMAttribute {
            local_name: "class".into(),
            value: "inactive".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(read(&owner), (7.0, None));
    owner.GetDocumentMut().SetAttribute(
        root,
        dom::persistent_document::DOMAttribute {
            local_name: "class".into(),
            value: "active".into(),
            ..Default::default()
        },
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(read(&owner), (7.0, Some(9.0)));
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
}

#[test]
fn production_container_query_reaches_translated_style_resolver_main_entry() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><section id=container><p id=target class=item></p></section>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#container{container-type:inline-size}.item{width:7px}\
         @container (width > 100px){.item{width:41px}}",
    ));
    let container = node(&owner, "container");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(engine
        .SetContainerSizes(&mut owner, [(container, 200.0, 50.0)])
        .unwrap());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let wide = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(wide.Width().Pixels(), 41.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    assert!(engine
        .SetContainerSizes(&mut owner, [(container, 80.0, 50.0)])
        .unwrap());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let narrow = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(narrow.Width().Pixels(), 7.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
}

#[test]
fn production_slotted_shadow_container_query_uses_flat_tree_ancestry_in_update() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><x-host id=host><p id=target class=item slot=named></p></x-host>",
    );
    let host = node(&owner, "host");
    let target = node(&owner, "target");
    let container = {
        let document = owner.GetDocumentMut();
        let shadow =
            document.AttachShadowRoot(host, dom::persistent_document::DOMShadowRootMode::kOpen);
        let container = document.CreateElement(
            dom::persistent_document::DOMNamespace::kHTML,
            "section".into(),
            vec![dom::persistent_document::DOMAttribute {
                local_name: "style".into(),
                value: "container-type:size".into(),
                ..Default::default()
            }],
        );
        let slot = document.CreateElement(
            dom::persistent_document::DOMNamespace::kHTML,
            "slot".into(),
            vec![dom::persistent_document::DOMAttribute {
                local_name: "name".into(),
                value: "named".into(),
                ..Default::default()
            }],
        );
        document.AppendChild(shadow, container);
        document.AppendChild(container, slot);
        assert_eq!(document.AssignedSlot(target), Some(slot));

        let initial = unsafe {
            &*layoutng_style::style::computed_style::ComputedStyle::GetInitialStyleSingleton()
        };
        let mut builder =
            layoutng_style::style::computed_style::ComputedStyleBuilder::from_style(initial);
        builder.SetContainerType(1);
        let native = foundation::Persistent::from_ptr(
            builder.TakeStyle() as *mut layoutng_style::style::computed_style::ComputedStyle
        );
        let mut resolved = dom::persistent_document::ResolvedNodeStyle::default();
        resolved.SetNativeStyle(native);
        document.SetResolvedStyle(container, resolved);
        container
    };
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".item{width:7px}@container (width > 100px){.item{width:41px}}",
    ));
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(engine
        .SetContainerSizes(&mut owner, [(container, 200.0, 50.0)])
        .unwrap());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let wide = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(wide.Width().Pixels(), 41.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    assert!(engine
        .SetContainerSizes(&mut owner, [(container, 80.0, 50.0)])
        .unwrap());
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let narrow = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert_eq!(narrow.Width().Pixels(), 7.0);
    let stats = engine.TranslatedRuleShadowStats();
    assert!(stats.translated_style_resolver_applied >= 1, "{stats:?}");
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
}

#[test]
fn update_uses_flat_tree_style_parents_for_slot_assignment_and_fallback() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><x-host id=host style='font-size:31px'>\
         <span id=assigned slot=named></span>\
         <span id=unassigned slot=missing><i id=unassigned-child></i></span>\
         </x-host>",
    );
    let host = node(&owner, "host");
    let assigned = node(&owner, "assigned");
    let unassigned = node(&owner, "unassigned");
    let unassigned_child = node(&owner, "unassigned-child");
    let (slot, fallback, fallback_child, shadow_child) = {
        let document = owner.GetDocumentMut();
        let shadow =
            document.AttachShadowRoot(host, dom::persistent_document::DOMShadowRootMode::kOpen);
        let shadow_child = document.CreateElementDefault(
            dom::persistent_document::DOMNamespace::kHTML,
            "section".into(),
        );
        let slot = document.CreateElement(
            dom::persistent_document::DOMNamespace::kHTML,
            "slot".into(),
            vec![
                dom::persistent_document::DOMAttribute {
                    local_name: "name".into(),
                    value: "named".into(),
                    ..Default::default()
                },
                dom::persistent_document::DOMAttribute {
                    local_name: "style".into(),
                    value: "font-size:23px".into(),
                    ..Default::default()
                },
            ],
        );
        let fallback = document
            .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "b".into());
        let fallback_child = document.CreateElementDefault(
            dom::persistent_document::DOMNamespace::kHTML,
            "small".into(),
        );
        document.AppendChild(fallback, fallback_child);
        document.AppendChild(slot, fallback);
        document.AppendChild(shadow, shadow_child);
        document.AppendChild(shadow, slot);
        assert_eq!(document.AssignedSlot(assigned), Some(slot));
        assert_eq!(document.AssignedSlot(unassigned), None);
        (slot, fallback, fallback_child, shadow_child)
    };
    let initial = crate::resolver::production_style_resolver::InitialStyleForDocument(
        &media(),
        crate::resolver::production_style_builder::ColorSchemeSettings::default(),
    );
    let initial_size = unsafe { &*initial.Get() }
        .GetFontDescription()
        .SpecifiedSize();
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(font_size(&owner, host), 31.0);
    assert_eq!(font_size(&owner, shadow_child), 31.0);
    assert_eq!(font_size(&owner, slot), 23.0);
    assert_eq!(font_size(&owner, assigned), 23.0);
    assert_eq!(font_size(&owner, unassigned), initial_size);
    assert_eq!(font_size(&owner, unassigned_child), initial_size);
    assert_eq!(font_size(&owner, fallback), initial_size);
    assert_eq!(font_size(&owner, fallback_child), initial_size);

    owner.GetDocumentMut().Remove(assigned);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(font_size(&owner, fallback), 23.0);
    assert_eq!(font_size(&owner, fallback_child), 23.0);
}

#[test]
fn update_consumes_ordered_manual_slot_owners_and_fallback_activation() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<!doctype html><x-host id=host style='font-size:31px'>\
         <span id=first></span><span id=second></span></x-host>",
    );
    let host = node(&owner, "host");
    let first = node(&owner, "first");
    let second = node(&owner, "second");
    let (primary, secondary, fallback) = {
        let document = owner.GetDocumentMut();
        let shadow = document.AttachShadowRootWithSlotAssignment(
            host,
            dom::persistent_document::DOMShadowRootMode::kOpen,
            dom::persistent_document::DOMSlotAssignmentMode::kManual,
        );
        let primary = document.CreateElement(
            dom::persistent_document::DOMNamespace::kHTML,
            "slot".into(),
            vec![dom::persistent_document::DOMAttribute {
                local_name: "style".into(),
                value: "font-size:23px".into(),
                ..Default::default()
            }],
        );
        let secondary = document.CreateElement(
            dom::persistent_document::DOMNamespace::kHTML,
            "slot".into(),
            vec![dom::persistent_document::DOMAttribute {
                local_name: "style".into(),
                value: "font-size:29px".into(),
                ..Default::default()
            }],
        );
        let fallback = document
            .CreateElementDefault(dom::persistent_document::DOMNamespace::kHTML, "b".into());
        document.AppendChild(primary, fallback);
        document.AppendChild(shadow, primary);
        document.AppendChild(shadow, secondary);
        assert!(document.AssignSlotNodes(primary, vec![second, first, second]));
        assert_eq!(document.ManuallyAssignedNodes(primary), [second, first]);
        assert_eq!(document.ManuallyAssignedSlot(first), Some(primary));
        assert_eq!(document.AssignedNodes(primary), [second, first]);
        (primary, secondary, fallback)
    };
    let initial = crate::resolver::production_style_resolver::InitialStyleForDocument(
        &media(),
        crate::resolver::production_style_builder::ColorSchemeSettings::default(),
    );
    let initial_size = unsafe { &*initial.Get() }
        .GetFontDescription()
        .SpecifiedSize();
    let mut engine = StyleEngine::new(&owner);

    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(font_size(&owner, first), 23.0);
    assert_eq!(font_size(&owner, second), 23.0);
    assert_eq!(font_size(&owner, fallback), initial_size);

    {
        let document = owner.GetDocumentMut();
        assert!(document.AssignSlotNodes(secondary, vec![first]));
        assert_eq!(document.ManuallyAssignedNodes(primary), [second]);
        assert_eq!(document.ManuallyAssignedNodes(secondary), [first]);
    }
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(font_size(&owner, first), 29.0);
    assert_eq!(font_size(&owner, second), 23.0);

    assert!(owner.GetDocumentMut().AssignSlotNodes(primary, Vec::new()));
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert_eq!(font_size(&owner, second), initial_size);
    assert_eq!(font_size(&owner, fallback), 23.0);
}

#[test]
fn production_animation_shorthand_names_composition_and_inheritance_reach_native_data() {
    use layoutng_style::style::css_timing_data::{
        CompositeOperation, PlaybackDirection, StyleTimeline, TimingFunction,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=parent style='animation:fade 2s steps(calc(3)) -250ms 2 alternate both paused,fade 1s linear;animation-composition:add,accumulate'><div id=target style='animation:inherit;animation-composition:inherit'></div></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "@keyframes fade{from{opacity:0}to{opacity:1}}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let style = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .native_style
        .clone();
    let native = unsafe { &*style.Get() };
    let animation = unsafe { &*native.Animations().Get() };
    assert_eq!(
        unsafe { &*animation.NameList()[0].Get() }.GetName().Utf8(),
        "fade"
    );
    assert_eq!(animation.DurationList(), &[Some(2.0), Some(1.0)]);
    assert_eq!(
        animation.GetComposition(1),
        CompositeOperation::kCompositeAccumulate
    );
    assert_eq!(
        animation.ConvertToTiming(0).direction,
        PlaybackDirection::ALTERNATE_NORMAL
    );
    assert!(matches!(
        &*animation.ConvertToTiming(0).timing_function,
        TimingFunction::Steps {
            number_of_steps: 3,
            ..
        }
    ));
    assert_eq!(
        animation.TimelineList(),
        &[StyleTimeline::Keyword(foundation::CSSValueID::kAuto)]
    );
    assert!(animation.RangeStartList()[0].is_none() && animation.RangeEndList()[0].is_none());
    set_attribute(
        &mut owner,
        target,
        "style",
        "animation:none;animation-composition:replace",
    );
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let updated = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    let updated = unsafe { &*updated.Animations().Get() };
    assert!(updated.NameList()[0].Get().is_null());
    assert_eq!(updated.DurationList(), &[None]);
    assert_eq!(
        updated.GetComposition(0),
        CompositeOperation::kCompositeReplace
    );
    assert_eq!(animation.DurationList(), &[Some(2.0), Some(1.0)]);
}

#[test]
fn production_grid_template_areas_shorthand_and_inheritance_reach_native_grid_data() {
    use foundation::String;
    use layoutng_style::style::computed_style_constants::GridAutoFlow;
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=parent style='display:grid;grid: auto-flow dense 12px / 1fr 40px'><div id=target style='grid:inherit'></div></div>",
    );
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target{grid-template:\"head head\" 20px \"main side\" / 2fr 30px!important}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let style = owner
        .GetDocument()
        .ResolvedStyleFor(target)
        .unwrap()
        .native_style
        .clone();
    let native = unsafe { &*style.Get() };
    let areas = unsafe { &*native.GridTemplateAreas().Get() };
    assert_eq!((areas.row_count, areas.column_count), (2, 2));
    assert_eq!(
        areas.implicit_named_grid_row_lines[&String::from("head-end")],
        vec![1]
    );
    assert_eq!(native.GetGridAutoFlow(), GridAutoFlow::kAutoFlowRowDense);
    assert_eq!(
        native
            .GridAutoRows()
            .RepeatTrackSize(0, 0)
            .MinTrackBreadth()
            .Pixels(),
        12.0
    );
    let columns = unsafe { &*native.SpecifiedGridTemplateColumns().Get() }.GetTrackList();
    assert_eq!(
        columns.RepeatTrackSize(0, 0).MaxTrackBreadth().FlexValue(),
        2.0
    );
    set_attribute(&mut owner, target, "style", "grid:none!important");
    engine.Update(&mut owner, &media(), &[]).unwrap();
    assert!(
        engine.Diagnostics().is_empty(),
        "{:?}",
        engine.Diagnostics()
    );
    let updated = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    assert!(
        updated.GridTemplateAreas().Get().is_null()
            && updated.SpecifiedGridTemplateColumns().Get().is_null()
            && updated.SpecifiedGridTemplateRows().Get().is_null()
    );
    assert_eq!(updated.GetGridAutoFlow(), GridAutoFlow::kAutoFlowRow);
    assert!(updated
        .GridAutoRows()
        .RepeatTrackSize(0, 0)
        .MinTrackBreadth()
        .IsAuto());
    assert_eq!(areas.row_count, 2);
}

#[test]
fn named_grid_lines_reach_native_track_list_and_grid_positions() {
    use foundation::String;

    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<div id=target></div>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        "#target { display:grid; grid-template-rows:1fr [line1] 2fr [line2] 1fr; \
         grid-row-start:line1; grid-row-end:line2 }",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");

    let native = unsafe {
        &*owner
            .GetDocument()
            .ResolvedStyleFor(target)
            .unwrap()
            .native_style
            .Get()
    };
    let rows = unsafe { &*native.SpecifiedGridTemplateRows().Get() };
    assert_eq!(rows.GetTrackList().TrackCountWithoutAutoRepeat(), 3);
    assert_eq!(rows.GetNamedGridLines()[&String::from("line1")], [1]);
    assert_eq!(rows.GetNamedGridLines()[&String::from("line2")], [2]);
    assert!(native.GridRowStart().IsNamedGridArea());
    assert_eq!(native.GridRowStart().NamedGridLine().Utf8(), "line1");
    assert!(native.GridRowEnd().IsNamedGridArea());
    assert_eq!(native.GridRowEnd().NamedGridLine().Utf8(), "line2");
}

#[test]
fn translated_element_resolution_retains_dynamic_pseudo_markers() {
    use layoutng_style::style::computed_style_constants::PseudoId;

    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<!doctype html><a id=target class=link></a>");
    owner.GetDocumentMut().AppendStyleSheet(crate::ParseCSS(
        ".link{color:#123456}.link::before{content:'marker'}",
    ));
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();

    let stats = engine.TranslatedRuleShadowStats();
    assert_eq!(stats.translated_style_resolver_fallbacks, 0, "{stats:?}");
    assert!(stats.translated_style_resolver_applied > 0, "{stats:?}");
    let style = owner.GetDocument().ResolvedStyleFor(target).unwrap();
    let native = unsafe { &*style.native_style.Get() };
    assert!(native.HasPseudoElementStyle(PseudoId::kPseudoIdBefore));
    assert_eq!(
        style.before.as_ref().map(|pseudo| pseudo.text.as_str()),
        Some("marker")
    );
}

use crate::{media_queries::MediaValuesCachedData, StyleEngine};
use foundation::{Color, EDisplay, EPosition};
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
#[test]
fn document_native_style_ua_author_inline_inheritance_and_media() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<html><body id=parent style='font-size:20px;color:red;white-space:pre'><span id=child style='width:30px;position:absolute;font-family:serif, sans-serif'></span><div id=hidden><b id=inside></b></div></body></html>");
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
fn unsupported_declarations_are_diagnostic_and_do_not_drop_valid_styles() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML(
        "<div id=target style='width:7px;transform:rotate(1deg);padding-left:9px'></div>",
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
    let mut owner=html::html_parser::ParseHTML("<html><head><style media='(min-width:700px)'>#target{width:23px;background-color:red}</style></head><body><div id=target></div></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><div id=target class=small><b id=descendant></b></div><section id=unrelated><b></b><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><section id=host><b id=target class=item></b><b id=plain></b></section><section id=unrelated><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><head><style id=sheet>#first{width:10px}</style></head><body><div id=first></div><div id=second></div><div id=unrelated><b></b><b></b></div></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><ul id=list><li id=first></li><li id=last></li></ul><section id=unrelated><b></b><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><div id=parent style='opacity:.1;display:none'><span id=child style='opacity:inherit'><b id=grandchild style='opacity:inherit'></b></span></div><section id=unrelated><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><div id=container></div><div id=target></div><section id=unrelated><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><section id=host><b id=first class=selected></b><b id=second class=selected></b></section><section id=unrelated><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<html><body><div id=first></div><div id=second></div><section id=unrelated><b></b><b></b></section></body></html>");
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
    let mut owner = html::html_parser::ParseHTML("<style>#target{display:block;color:red;font-size:20px} #target::before{content:'wrong';color:blue} .host::before{content:attr(data-label) ' \\41';color:green} #target::after{content:'';display:contents} #target::first-letter{font-size:40px} input::placeholder{color:blue;font-style:italic}</style><div id=target class=host data-label=hello style='color:red;content:none'>Alpha</div><input id=control placeholder=hint>");
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
}

#[test]
fn production_pseudo_cache_retains_suppressed_attr_content_and_invalidates_it() {
    use layoutng_style::style::{
        computed_style_constants::PseudoId, content_data::TextContentData,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<style>.host::before{content:attr(data-label);display:none} .host.show::before{display:block} .host::after{content:none}</style><div id=target class=host data-label=one></div>");
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
    let mut owner = html::html_parser::ParseHTML("<style>.host{display:contents}.host::before{content:'yes'}.host::first-letter{font-size:40px}</style><div id=contents class=host>Text</div><section style='display:none'><div id=suppressed class=host>Text</div></section>");
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
    let mut owner = html::html_parser::ParseHTML("<style>.host{display:block;color:red} .host::before{content:attr(data-label) ' \\41'} .hidden::before{display:none} .host::after{content:none} .ancestor.on .host::after{content:'tail'}</style><section id=ancestor class=ancestor><div id=target class=host data-label=one>Text</div></section><div id=unrelated class=host data-label=other>Other</div>");
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
fn production_pseudo_sheet_removal_and_unsupported_requests_do_not_fake_content() {
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner = html::html_parser::ParseHTML("<style id=sheet>#target::before{content:'ok'} #target::after{content:'fallback';content:counter(item, symbols('x'))} #target::marker{color:red}</style><div id=target></div>");
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
    let mut owner = html::html_parser::ParseHTML("<main class=outer id=root><p id=direct class=item></p><section class=inner><p id=nested class=item></p></section><aside class=stop><p id=limited class=item></p><section class=inner><p id=blocked class=item></p></section></aside></main><p id=outside class=item></p>");
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
    let mut owner = html::html_parser::ParseHTML("<main id=root class=active><style>@scope{.item{width:7px}} @scope (.active) to (.stop){.item{height:9px}}</style><section id=limit><p id=target class=item></p></section></main><p id=outside class=item></p>");
    // An ownerless implicit scope must never become a document-wide rule.
    owner
        .GetDocumentMut()
        .AppendStyleSheet(crate::ParseCSS("@scope{.item{width:99px}}"));
    let root = node(&owner, "root");
    let limit = node(&owner, "limit");
    let target = node(&owner, "target");
    let mut engine = StyleEngine::new(&owner);
    engine.Update(&mut owner, &media(), &[]).unwrap();
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
fn production_animation_shorthand_names_composition_and_inheritance_reach_native_data() {
    use layoutng_style::style::css_timing_data::{
        CompositeOperation, PlaybackDirection, StyleTimeline, TimingFunction,
    };
    let _heap = foundation::LayoutHeapScope::new();
    let mut owner=html::html_parser::ParseHTML("<div id=parent style='animation:fade 2s steps(calc(3)) -250ms 2 alternate both paused,fade 1s linear;animation-composition:add,accumulate'><div id=target style='animation:inherit;animation-composition:inherit'></div></div>");
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
    let mut owner=html::html_parser::ParseHTML("<div id=parent style='display:grid;grid: auto-flow dense 12px / 1fr 40px'><div id=target style='grid:inherit'></div></div>");
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

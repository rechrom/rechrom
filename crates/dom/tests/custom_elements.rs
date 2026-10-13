use dom::custom_elements::{
    CustomElementCallbacks, CustomElementDefinitionId, CustomElementDescriptor,
    CustomElementReaction as Reaction, CustomElementRegistrationError as RegistrationError,
    CustomElementState as State, IsValidCustomElementName,
};
use dom::persistent_document::{DOMAttribute, DOMNamespace, DOMShadowRootMode};

fn attribute(name: &str, value: &str) -> DOMAttribute {
    DOMAttribute {
        local_name: name.into(),
        value: value.into(),
        ..Default::default()
    }
}

fn register(d: &mut dom::Document, name: &str) -> CustomElementDefinitionId {
    d.RegisterCustomElementDefinition(
        CustomElementDescriptor {
            name: name.into(),
            local_name: name.into(),
        },
        vec!["first".into(), "second".into()],
        CustomElementCallbacks {
            connected: true,
            disconnected: true,
            attribute_changed: true,
            ..Default::default()
        },
    )
    .unwrap()
}

fn finish_next_upgrade(d: &mut dom::Document, element: usize, id: CustomElementDefinitionId) {
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Upgrade { element, id })
    );
    assert!(d.BeginCustomElementUpgrade(element, id));
    assert_eq!(d.CustomElementState(element), State::Precustomized);
    assert!(d.CustomElementDefinitionForElement(element).is_none());
    d.CompleteCustomElementUpgrade(element, id, true);
    assert_eq!(d.CustomElementState(element), State::Custom);
}

#[test]
fn registration_failure_is_atomic_and_preserves_original_definition() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let id = register(d, "x-atomic");
    let before = d
        .CustomElementDefinitionForName("x-atomic")
        .unwrap()
        .clone();
    let duplicate = d.RegisterCustomElementDefinition(
        CustomElementDescriptor {
            name: "x-atomic".into(),
            local_name: "x-atomic".into(),
        },
        vec!["replacement".into()],
        CustomElementCallbacks::default(),
    );
    assert_eq!(duplicate, Err(RegistrationError::DuplicateName));
    assert_eq!(d.CustomElementDefinitionForName("x-atomic"), Some(&before));
    let unsupported = d.RegisterCustomElementDefinition(
        CustomElementDescriptor {
            name: "x-button".into(),
            local_name: "button".into(),
        },
        Vec::new(),
        CustomElementCallbacks::default(),
    );
    assert_eq!(
        unsupported,
        Err(RegistrationError::CustomizedBuiltInUnsupported)
    );
    assert!(d.CustomElementDefinitionForName("x-button").is_none());
    assert_eq!(register(d, "x-next").0, id.0 + 1);
    assert!(!d.HasPendingCustomElementReactions());
}

#[test]
fn custom_element_names_use_current_dom_local_name_rules() {
    for name in ["x-a", "x-😀", "x-:part", "x-=value", "x-!", "x-\u{b}"] {
        assert!(IsValidCustomElementName(name), "{name:?}");
    }
    for name in [
        "",
        "div",
        "-x",
        "X-tag",
        "x-Tag",
        "x- name",
        "x-\t",
        "x-/",
        "x->",
        "x-\0",
        "font-face",
    ] {
        assert!(!IsValidCustomElementName(name), "{name:?}");
    }
}

#[test]
fn initial_callbacks_follow_attribute_order_before_connected_and_next_element() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let first = d.CreateElement(
        DOMNamespace::kHTML,
        "x-order".into(),
        vec![
            attribute("second", "2"),
            attribute("ignored", "0"),
            attribute("first", "1"),
        ],
    );
    let second = d.CreateElementDefault(DOMNamespace::kHTML, "x-order".into());
    d.AppendChild(d.Root(), first);
    d.AppendChild(d.Root(), second);
    let id = register(d, "x-order");
    finish_next_upgrade(d, first, id);
    for (name, value) in [("second", "2"), ("first", "1")] {
        assert_eq!(
            d.PopCustomElementReaction(),
            Some(Reaction::AttributeChanged {
                element: first,
                id,
                name: name.into(),
                old_value: None,
                new_value: Some(value.into()),
                namespace: None,
            })
        );
    }
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected { element: first, id })
    );
    finish_next_upgrade(d, second, id);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected {
            element: second,
            id
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn failed_upgrade_clears_callbacks_and_never_retries() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let element = d.CreateElement(
        DOMNamespace::kHTML,
        "x-failure".into(),
        vec![attribute("first", "1")],
    );
    d.AppendChild(d.Root(), element);
    let id = register(d, "x-failure");
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Upgrade { element, id })
    );
    assert!(d.BeginCustomElementUpgrade(element, id));
    d.CompleteCustomElementUpgrade(element, id, false);
    assert_eq!(d.CustomElementState(element), State::Failed);
    d.EnqueueCustomElementUpgradeSubtree(element);
    d.SetAttribute(element, attribute("first", "2"));
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(!d.BeginCustomElementUpgrade(element, id));
}

#[test]
fn constructor_mutations_do_not_create_additional_lifecycle_callbacks() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let element = d.CreateElement(
        DOMNamespace::kHTML,
        "x-constructor".into(),
        vec![attribute("first", "before")],
    );
    let id = register(d, "x-constructor");
    d.EnqueueCustomElementUpgradeSubtree(element);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Upgrade { element, id })
    );
    assert!(d.BeginCustomElementUpgrade(element, id));
    d.SetAttribute(element, attribute("first", "during"));
    d.AppendChild(d.Root(), element);
    d.CompleteCustomElementUpgrade(element, id, true);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::AttributeChanged {
            element,
            id,
            name: "first".into(),
            old_value: None,
            new_value: Some("before".into()),
            namespace: None,
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn attribute_mutations_include_equal_values_namespace_and_removal() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let element = d.CreateElementDefault(DOMNamespace::kHTML, "x-attributes".into());
    let id = register(d, "x-attributes");
    d.EnqueueCustomElementUpgradeSubtree(element);
    finish_next_upgrade(d, element, id);
    let namespaced = DOMAttribute {
        namespace_uri: "urn:test".into(),
        prefix: "t".into(),
        ..attribute("first", "value")
    };
    d.SetAttribute(element, namespaced.clone());
    d.SetAttribute(element, namespaced);
    d.RemoveAttribute(element, "first", "urn:test");
    d.RemoveAttribute(element, "first", "urn:test");
    for (old, new) in [
        (None, Some("value")),
        (Some("value"), Some("value")),
        (Some("value"), None),
    ] {
        assert_eq!(
            d.PopCustomElementReaction(),
            Some(Reaction::AttributeChanged {
                element,
                id,
                name: "first".into(),
                old_value: old.map(str::to_owned),
                new_value: new.map(str::to_owned),
                namespace: Some("urn:test".into()),
            })
        );
    }
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn registration_and_explicit_upgrade_enter_closed_author_roots_only() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let shadow = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let shadow_child = d.CreateElementDefault(DOMNamespace::kHTML, "x-shadow".into());
    let light_child = d.CreateElementDefault(DOMNamespace::kHTML, "x-shadow".into());
    d.AppendChild(shadow, shadow_child);
    d.AppendChild(host, light_child);
    let id = register(d, "x-shadow");
    assert!(
        !d.HasPendingCustomElementReactions(),
        "registration does not upgrade detached candidates"
    );
    d.EnqueueCustomElementUpgradeSubtree(host);
    finish_next_upgrade(d, shadow_child, id);
    finish_next_upgrade(d, light_child, id);
    assert_eq!(d.PopCustomElementReaction(), None);
    d.AppendChild(d.Root(), host);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected {
            element: shadow_child,
            id
        })
    );
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected {
            element: light_child,
            id
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
    d.Remove(host);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Disconnected {
            element: shadow_child,
            id
        })
    );
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Disconnected {
            element: light_child,
            id
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
    let ua_host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let ua = d.AttachShadowRoot(ua_host, DOMShadowRootMode::kUserAgent);
    let ua_child = d.CreateElementDefault(DOMNamespace::kHTML, "x-shadow".into());
    d.AppendChild(ua, ua_child);
    d.EnqueueCustomElementUpgradeSubtree(ua_host);
    assert_eq!(d.PopCustomElementReaction(), None);
    assert_eq!(d.CustomElementState(ua_child), State::Undefined);
}

#[test]
fn connected_registration_order_is_shadow_before_light_dom() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let light = d.CreateElementDefault(DOMNamespace::kHTML, "x-candidate".into());
    let root = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let shadow = d.CreateElementDefault(DOMNamespace::kHTML, "x-candidate".into());
    d.AppendChild(host, light);
    d.AppendChild(root, shadow);
    d.AppendChild(d.Root(), host);
    let id = register(d, "x-candidate");
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Upgrade {
            element: shadow,
            id
        })
    );
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Upgrade { element: light, id })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn nested_scopes_do_not_drain_unrelated_outer_elements() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let first = d.CreateElementDefault(DOMNamespace::kHTML, "x-scope".into());
    let second = d.CreateElementDefault(DOMNamespace::kHTML, "x-scope".into());
    d.AppendChild(d.Root(), first);
    d.AppendChild(d.Root(), second);
    d.PushCustomElementReactionScope();
    let id = register(d, "x-scope");
    assert!(
        !d.PopCustomElementReactionScope(),
        "cannot discard pending work"
    );
    finish_next_upgrade(d, first, id);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected { element: first, id })
    );
    d.PushCustomElementReactionScope();
    d.SetAttribute(first, attribute("first", "nested"));
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::AttributeChanged {
            element: first,
            id,
            name: "first".into(),
            old_value: None,
            new_value: Some("nested".into()),
            namespace: None,
        })
    );
    assert_eq!(
        d.PopCustomElementReaction(),
        None,
        "second element belongs to the outer scope"
    );
    assert!(d.PopCustomElementReactionScope());
    finish_next_upgrade(d, second, id);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected {
            element: second,
            id
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(d.PopCustomElementReactionScope());
    assert!(!d.PopCustomElementReactionScope());
}

#[test]
fn registry_does_not_apply_to_foreign_namespaces_or_inert_documents() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    register(d, "x-scope");
    let foreign = d.CreateElementDefault(DOMNamespace::kSVG, "x-scope".into());
    assert_eq!(d.CustomElementState(foreign), State::Uncustomized);
    assert!(d.LookupCustomElementDefinition(foreign).is_none());
    let inert = d.CreateHTMLDocument(None);
    let element = d.CreateElementDefault(DOMNamespace::kHTML, "x-scope".into());
    d.AppendChild(inert, element);
    assert_eq!(d.CustomElementState(element), State::Undefined);
    assert!(d.LookupCustomElementDefinition(element).is_none());
    d.EnqueueCustomElementUpgradeSubtree(inert);
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn inserting_an_undefined_subtree_upgrades_newly_connected_candidates() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let id = register(d, "x-inserted");
    let host = d.CreateElementDefault(DOMNamespace::kHTML, "div".into());
    let shadow = d.AttachShadowRoot(host, DOMShadowRootMode::kClosed);
    let element = d.CreateElementDefault(DOMNamespace::kHTML, "x-inserted".into());
    d.AppendChild(shadow, element);
    assert!(!d.HasPendingCustomElementReactions());
    d.AppendChild(d.Root(), host);
    finish_next_upgrade(d, element, id);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected { element, id })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
}

#[test]
fn nested_scope_drains_existing_reactions_for_the_same_element_in_order() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let element = d.CreateElement(
        DOMNamespace::kHTML,
        "x-shared-queue".into(),
        vec![attribute("first", "1"), attribute("second", "2")],
    );
    d.AppendChild(d.Root(), element);
    d.PushCustomElementReactionScope();
    let id = register(d, "x-shared-queue");
    finish_next_upgrade(d, element, id);
    assert!(matches!(
        d.PopCustomElementReaction(),
        Some(Reaction::AttributeChanged { name, .. }) if name == "first"
    ));
    // Simulate setAttribute called inside the first attribute callback. Blink
    // shares this element's remaining reactions across nested element queues.
    d.PushCustomElementReactionScope();
    d.SetAttribute(element, attribute("first", "nested"));
    assert!(matches!(
        d.PopCustomElementReaction(),
        Some(Reaction::AttributeChanged { name, .. }) if name == "second"
    ));
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected { element, id })
    );
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::AttributeChanged {
            element,
            id,
            name: "first".into(),
            old_value: Some("1".into()),
            new_value: Some("nested".into()),
            namespace: None,
        })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(d.PopCustomElementReactionScope());
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(d.PopCustomElementReactionScope());
}

#[test]
fn backup_scheduling_signal_survives_an_empty_nested_scope() {
    let mut owner = dom::DOM::new();
    let d = owner.GetDocumentMut();
    let element = d.CreateElementDefault(DOMNamespace::kHTML, "x-backup".into());
    d.AppendChild(d.Root(), element);
    let id = register(d, "x-backup");
    assert!(d.HasPendingCustomElementBackupReactions());
    assert!(!d.HasActiveCustomElementReactionScope());
    d.PushCustomElementReactionScope();
    assert!(d.HasActiveCustomElementReactionScope());
    assert!(!d.HasPendingCustomElementReactions());
    assert!(d.HasPendingCustomElementBackupReactions());
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(d.PopCustomElementReactionScope());
    finish_next_upgrade(d, element, id);
    assert_eq!(
        d.PopCustomElementReaction(),
        Some(Reaction::Connected { element, id })
    );
    assert_eq!(d.PopCustomElementReaction(), None);
    assert!(!d.HasPendingCustomElementBackupReactions());
}

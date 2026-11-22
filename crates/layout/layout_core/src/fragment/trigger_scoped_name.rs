#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, Member, ScopedCSSName};

use super::super::layout_node_metadata::Element;
use super::super::naming_scope::NamingScope;

pub type TriggerScopedName = NamingScope;

pub fn ToTriggerScopedName(
    name: &ScopedCSSName,
    originating_element: &Element,
) -> *mut TriggerScopedName {
    let scope_element = TriggerScopedName::FindScopeElement(name, originating_element, |style| {
        style.TriggerScope()
    });
    MakeGarbageCollected(TriggerScopedName::new(name, scope_element))
}

pub type TriggerScopedNameMap =
    foundation::GCedHeapHashMap<Member<TriggerScopedName>, Member<Element>>;

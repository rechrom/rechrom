#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, Member, ScopedCSSName};

use super::layout_node_metadata::Element;
use super::naming_scope::NamingScope;

// cpp: layoutng/internal/trigger_scoped_name.h:19-19
pub type TriggerScopedName = NamingScope;

// cpp: layoutng/internal/trigger_scoped_name.h:21-21
// cpp: layoutng/internal/naming_scope.cc:80-87
pub fn ToTriggerScopedName(
    name: &ScopedCSSName,
    originating_element: &Element,
) -> *mut TriggerScopedName {
    let scope_element = TriggerScopedName::FindScopeElement(name, originating_element, |style| {
        style.TriggerScope()
    });
    MakeGarbageCollected(TriggerScopedName::new(name, scope_element))
}

// cpp: layoutng/internal/trigger_scoped_name.h:25-26
pub type TriggerScopedNameMap =
    foundation::GCedHeapHashMap<Member<TriggerScopedName>, Member<Element>>;

//! Trigger scoped names: the value and map types are fragment data
//! (`layout_fragment`); building a name walks the originating Element.

#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, ScopedCSSName};

use super::layout_node_metadata::Element;
use super::naming_scope::NamingScopeExt;

pub use layout_fragment::internal::trigger_scoped_name::*;

pub fn ToTriggerScopedName(
    name: &ScopedCSSName,
    originating_element: &Element,
) -> *mut TriggerScopedName {
    let scope_element = TriggerScopedName::FindScopeElement(name, originating_element, |style| {
        style.TriggerScope()
    });
    MakeGarbageCollected(TriggerScopedName::new(name, scope_element.cast()))
}

#![allow(non_snake_case)]

use foundation::{MakeGarbageCollected, ScopedCSSName};

use super::layout_node_metadata::Element;
use super::layout_object::LayoutObject;
use super::naming_scope::NamingScope;

// cpp: layoutng/internal/anchor_scope.h:15-15
pub type AnchorScopedName = NamingScope;

// cpp: layoutng/internal/anchor_scope.h:17-17
// cpp: layoutng/internal/naming_scope.cc:71-78
pub fn ToAnchorScopedName(
    name: &ScopedCSSName,
    layout_object: &LayoutObject,
) -> *mut AnchorScopedName {
    let node = layout_object.GetNode();
    assert!(!node.is_null());
    assert!(unsafe { &*node }.IsElementNode());
    let element = unsafe { &*node.cast::<Element>() };
    let scope_element =
        AnchorScopedName::FindScopeElement(name, element, |style| style.AnchorScope());
    MakeGarbageCollected(AnchorScopedName::new(name, scope_element))
}

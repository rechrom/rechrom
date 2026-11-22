//! Scoped-name resolution. `NamingScope` is fragment data (`layout_fragment`);
//! finding the scope element walks the Element's ancestors and is this base
//! extension trait.

#![allow(non_snake_case, non_upper_case_globals)]

use foundation::{ScopedCSSName, StyleNameScope};
use layout_style::style::computed_style::ComputedStyle;

use super::layout_node_metadata::Element;

pub use layout_fragment::internal::naming_scope::*;

/// NamingScope behaviour that dereferences DOM projection Elements.
pub trait NamingScopeExt {
    fn FindScopeElement<F>(
        name: &ScopedCSSName,
        start_element: &Element,
        get_scope: F,
    ) -> *const Element
    where
        F: for<'a> Fn(&'a ComputedStyle) -> &'a StyleNameScope;
}

impl NamingScopeExt for NamingScope {
    // cpp: layoutng/internal/naming_scope.cc:45-69
    #[inline(never)]
    fn FindScopeElement<F>(
        name: &ScopedCSSName,
        start_element: &Element,
        get_scope: F,
    ) -> *const Element
    where
        F: for<'a> Fn(&'a ComputedStyle) -> &'a StyleNameScope,
    {
        let mut element = start_element as *const Element;
        while !element.is_null() {
            let node = unsafe { &(*element).container.node };
            let style = node.GetComputedStyle();
            if !style.is_null() {
                let scope = get_scope(unsafe { &*style });
                if IsWithinScope(name, scope) {
                    return element;
                }
            }
            element = node.parentElement();
        }
        std::ptr::null()
    }
}

// cpp: layoutng/internal/naming_scope.cc:19-36
fn IsWithinScope(lookup_name: &ScopedCSSName, scope: &StyleNameScope) -> bool {
    if scope.IsNone() {
        return false;
    }
    if scope.IsAll() {
        return scope.AllTreeScope() == lookup_name.GetTreeScope();
    }
    let scoped_names = scope.Names();
    assert!(!scoped_names.is_null());
    for scoped_name in unsafe { &*scoped_names }.GetNames() {
        if unsafe { &*scoped_name.Get() } == lookup_name {
            return true;
        }
    }
    false
}

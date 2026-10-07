#![allow(non_snake_case, non_upper_case_globals)]

use std::hash::{Hash, Hasher};

use foundation::{
    AddIntToHash, AtomicString, GetHash, Member, ScopedCSSName, StyleNameScope, ValuesEquivalent,
    Visitor,
};
use layoutng_style::style::computed_style::ComputedStyle;

use super::layout_node_metadata::Element;

// cpp: layoutng/internal/naming_scope.h:36-70
pub struct NamingScope {
    name_: Member<ScopedCSSName>,
    scope_element_: Member<Element>,
}

impl NamingScope {
    // cpp: layoutng/internal/naming_scope.h:38-39
    pub fn new(name: &ScopedCSSName, scope_element: *const Element) -> Self {
        Self {
            name_: Member::from_ptr(name as *const ScopedCSSName as *mut ScopedCSSName),
            scope_element_: Member::from_ptr(scope_element as *mut Element),
        }
    }

    // cpp: layoutng/internal/naming_scope.h:41-42
    pub fn GetName(&self) -> &AtomicString {
        unsafe { &*self.name_.Get() }.GetName()
    }

    pub fn GetScopedName(&self) -> *const ScopedCSSName {
        self.name_.Get()
    }

    // cpp: layoutng/internal/naming_scope.h:49-53
    pub fn GetHash(&self) -> u32 {
        let mut hash = unsafe { &*self.name_.Get() }.GetHash();
        AddIntToHash(&mut hash, GetHash(self.scope_element_.Get()));
        hash
    }

    // cpp: layoutng/internal/naming_scope.cc:40-43
    pub fn Trace(&self, visitor: &mut Visitor) {
        visitor.Trace(&self.name_);
        visitor.Trace(&self.scope_element_);
    }

    // cpp: layoutng/internal/naming_scope.cc:45-69
    pub fn FindScopeElement<F>(
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

// cpp: layoutng/internal/naming_scope.h:44-47
impl PartialEq for NamingScope {
    fn eq(&self, other: &Self) -> bool {
        ValuesEquivalent(&self.name_, &other.name_)
            && self.scope_element_.Get() == other.scope_element_.Get()
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

// C++ specializes HashTraits<Member<NamingScope>> for value-based lookup.
// A newtype makes that choice explicit without changing the pointer identity
// behavior of Member<T> elsewhere. Const and mutable Member specializations
// share this Rust wrapper because Rust borrows carry constness.
// cpp: layoutng/internal/naming_scope.h:72-88
pub struct NamingScopeKey(pub Member<NamingScope>);

impl NamingScopeKey {
    pub const kSafeToCompareToEmptyOrDeleted: bool = false;
}

impl PartialEq for NamingScopeKey {
    fn eq(&self, other: &Self) -> bool {
        ValuesEquivalent(&self.0, &other.0)
    }
}

impl Eq for NamingScopeKey {}

impl Hash for NamingScopeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if self.0.Get().is_null() {
            0u32.hash(state);
        } else {
            unsafe { &*self.0.Get() }.GetHash().hash(state);
        }
    }
}

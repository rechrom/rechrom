//! Scoped-name identity for style invalidation.
//!
//! The scope element is an opaque Element handle. Resolving it by walking
//! the element's ancestors (`NamingScope::FindScopeElement`,
//! naming_scope.cc:19-36,45-69) is the base extension trait `NamingScopeExt`
//! (`layout_core/src/internal/naming_scope.rs`).

#![allow(non_snake_case, non_upper_case_globals)]

use std::hash::{Hash, Hasher};

use foundation::{AddIntToHash, AtomicString, GetHash, Member, ScopedCSSName, ValuesEquivalent, Visitor};

use crate::opaque::OpaqueNode;

// cpp: layoutng/internal/naming_scope.h:36-70
pub struct NamingScope {
    name_: Member<ScopedCSSName>,
    scope_element_: Member<OpaqueNode>,
}

impl NamingScope {
    // cpp: layoutng/internal/naming_scope.h:38-39
    pub fn new(name: &ScopedCSSName, scope_element: *const OpaqueNode) -> Self {
        Self {
            name_: Member::from_ptr(name as *const ScopedCSSName as *mut ScopedCSSName),
            scope_element_: Member::from_ptr(scope_element.cast_mut()),
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
}

// cpp: layoutng/internal/naming_scope.h:44-47
impl PartialEq for NamingScope {
    fn eq(&self, other: &Self) -> bool {
        ValuesEquivalent(&self.name_, &other.name_)
            && self.scope_element_.Get() == other.scope_element_.Get()
    }
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

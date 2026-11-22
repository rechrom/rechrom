//! Compatibility facade for invalidation-owned scoped-name resolution.

#![allow(non_snake_case, non_upper_case_globals)]

#[path = "invalidation/naming_scope.rs"]
mod layout_invalidation_naming_scope;

pub use layout_invalidation_naming_scope::*;

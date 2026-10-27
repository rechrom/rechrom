//! Compatibility facade for invalidation-owned scoped-name resolution.

#![allow(non_snake_case, non_upper_case_globals)]

#[path = "../../../layout_invalidation/src/native_naming_scope.rs"]
mod layout_invalidation_native_naming_scope;

pub use layout_invalidation_native_naming_scope::*;

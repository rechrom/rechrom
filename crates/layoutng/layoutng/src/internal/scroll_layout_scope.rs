//! Compatibility facade for fragment-owned scroll layout scopes.

#![allow(non_snake_case)]

#[path = "../../../layout_fragment/src/native_scroll_layout_scope.rs"]
mod layout_fragment_native_scroll_layout_scope;

pub use layout_fragment_native_scroll_layout_scope::*;

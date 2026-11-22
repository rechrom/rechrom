#![allow(non_snake_case)]

use super::naming_scope::NamingScope;

// cpp: layoutng/internal/anchor_scope.h:15-15
pub type AnchorScopedName = NamingScope;

// `ToAnchorScopedName` (anchor_scope.h:17, naming_scope.cc:71-78) reads the
// LayoutObject's node and lives in base (`layout_core/src/internal/anchor_scope.rs`).

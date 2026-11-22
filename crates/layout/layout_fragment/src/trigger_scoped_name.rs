#![allow(non_snake_case)]

use foundation::Member;

use super::super::naming_scope::NamingScope;
use crate::opaque::OpaqueNode;

pub type TriggerScopedName = NamingScope;

// Trigger name -> owning Element (an opaque handle). `ToTriggerScopedName`
// walks the originating element's ancestors and lives in base
// (`layout_core/src/internal/trigger_scoped_name.rs`).
pub type TriggerScopedNameMap =
    foundation::GCedHeapHashMap<Member<TriggerScopedName>, Member<OpaqueNode>>;

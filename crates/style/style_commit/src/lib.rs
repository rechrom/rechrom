#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Commit of resolved `ComputedStyle` and deferred post-style effects.
//!
//! Commit owns update application order and receives its lifecycle phase as an
//! injected guard. It does not parse, index rules, walk/match selectors,
//! resolve the cascade, or reach backward into the Style orchestrator.

pub mod post_style_update_scope;
pub mod resolved_style_commit;

pub use resolved_style_commit::{CommitResolvedStyles, ResolvedStyleCommitBackend};

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Flat-tree traversal state and style-recalc propagation.
//!
//! The package carries propagation decisions over a caller-supplied tree
//! backend. It does not own DOM nodes, selector indexes, cascade state,
//! ComputedStyle commits, or the document lifecycle.

pub mod flat_tree_walk;
pub mod style_recalc_change;
pub mod style_recalc_context;

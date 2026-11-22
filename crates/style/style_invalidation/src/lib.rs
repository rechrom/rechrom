// Identifiers keep Chromium's C++ names (methods, enum constants) for source alignment.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod invalidation_dom;
pub mod invalidation_flags;
pub mod invalidation_set;
pub mod node_invalidation_sets;
pub mod pending_invalidations;
pub mod rule_invalidation_data_visitor;
pub mod selector_pre_match;
pub mod style_invalidator;

#[cfg(test)]
mod deferred_invalidation_test;

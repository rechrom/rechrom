#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Rule-index ownership and update algorithms.
//!
//! This crate owns retained stylesheet/rule-set indexing state. It consumes
//! parsed model values and exposes lookup/update structures; it does not walk
//! the DOM, match selectors, run the cascade, commit styles, or orchestrate a
//! document lifecycle.

pub use style_model::media_value_change;

pub mod active_style_sheets;
pub mod cascade_layer;
pub mod cascade_layer_map;
pub mod cascade_layered;
pub mod fixed_size_cache;
pub mod font_feature_values_registry;
pub mod media_query_result;
pub mod rule_set;
pub mod seeker;
pub mod style_resolver_utils;
pub mod style_sheet_contents;

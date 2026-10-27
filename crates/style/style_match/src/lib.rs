#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

//! Selector matching state and retained match results.
//!
//! This layer consumes rule indexes plus caller-supplied tree access and emits
//! match results. It does not apply the cascade or commit ComputedStyle.

pub use style_model::valid_property_filter;
pub use style_model::{css_resource_fetch_restriction, css_selector, css_selector_list};

pub mod parser {
    pub use style_parser::*;
}

pub mod cascade_origin;
pub mod element_rule_collector;
pub mod match_flags;
pub mod match_result;
pub mod media_query_result;
pub mod selector_checker;
pub mod selector_filter;
pub mod selector_query;
